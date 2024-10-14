use super::utils::R2Location;
use async_trait::async_trait;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::ObjectCannedAcl;
use aws_sdk_s3::Client;
use log::info;
use reqwest::Client as ReqwestClient;
use std::error::Error;
use tokio::io::AsyncReadExt;

pub struct ObjectOutput {
    body: ByteStream,
}

// R2ClientTrait definition
#[async_trait]
pub trait R2ClientTrait {
    async fn put_object(
        &self,
        bucket: String,
        key: String,
        body: Vec<u8>,
    ) -> Result<(), Box<dyn Error>>;

    async fn get_object(&self, bucket: String, key: String)
        -> Result<ObjectOutput, Box<dyn Error>>;

    async fn list_objects(
        &self,
        bucket: String,
        prefix: String,
    ) -> Result<Vec<aws_sdk_s3::types::Object>, Box<dyn Error>>;
}

// R2Client implementation of R2ClientTrait
#[async_trait]
impl R2ClientTrait for Client {
    async fn put_object(
        &self,
        bucket: String,
        key: String,
        body: Vec<u8>,
    ) -> Result<(), Box<dyn Error>> {
        self.put_object()
            .bucket(bucket)
            .key(key)
            .body(body.into())
            .acl(ObjectCannedAcl::Private)
            .send()
            .await?;
        Ok(())
    }

    async fn get_object(
        &self,
        bucket: String,
        key: String,
    ) -> Result<ObjectOutput, Box<dyn Error>> {
        let resp = self.get_object().bucket(bucket).key(key).send().await?;
        Ok(ObjectOutput { body: resp.body })
    }

    async fn list_objects(
        &self,
        bucket: String,
        prefix: String,
    ) -> Result<Vec<aws_sdk_s3::types::Object>, Box<dyn Error>> {
        let resp = self
            .list_objects_v2()
            .bucket(bucket)
            .prefix(prefix)
            .send()
            .await?;
        Ok(resp.contents().to_vec())
    }
}

pub async fn sync_local_to_r2(
    client: &dyn R2ClientTrait,
    bucket_name: &str,
    dir_path: &str,
    key_prefix: &str,
    public_domain: &Option<String>,
) -> Result<(), Box<dyn Error>> {
    let paths = std::fs::read_dir(dir_path)?;

    for path in paths {
        let path = path?.path();
        if path.is_file() {
            let file_name = path.file_name().unwrap().to_str().unwrap();
            let key = if key_prefix.is_empty() {
                file_name.to_string()
            } else {
                format!("{}/{}", key_prefix, file_name)
            };

            let mut file = tokio::fs::File::open(&path).await?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).await?;

            if let Some(public_domain) = public_domain {
                let public_url = format!("https://{}/{}", public_domain, key);

                // Access the public URL and check etag as md5 hash and length
                let reqwest_client = ReqwestClient::new();
                let resp = reqwest_client.head(&public_url).send().await?;

                if resp.status().is_success() {
                    let etag = resp.headers().get("etag").unwrap().to_str().unwrap();
                    let content_length = resp
                        .headers()
                        .get("content-length")
                        .unwrap()
                        .to_str()
                        .unwrap();
                    let etag = etag.trim_matches('"');
                    let content_length = content_length.parse::<usize>()?;

                    let md5 = md5::compute(&buffer);
                    let md5 = format!("{:x}", md5);

                    if etag == md5 && content_length == buffer.len() {
                        info!("Skip identical file: {}", key);
                        continue;
                    }
                }
            }

            client
                .put_object(bucket_name.to_string(), key.clone(), buffer)
                .await?;
            info!("Uploaded: {}", key);
        }
    }
    Ok(())
}

pub async fn sync_r2_to_local(
    client: &dyn R2ClientTrait,
    bucket_name: &str,
    key_prefix: &str,
    local_dir: &str,
) -> Result<(), Box<dyn Error>> {
    let objects = client
        .list_objects(bucket_name.to_string(), key_prefix.to_string())
        .await?;

    for object in objects {
        let key = object.key().unwrap();
        let file_name = key.strip_prefix(key_prefix).unwrap_or(key);
        let local_path = format!("{}/{}", local_dir, file_name);

        let resp = client
            .get_object(bucket_name.to_string(), key.to_string())
            .await?;

        let mut file = tokio::fs::File::create(&local_path).await?;
        let mut stream = resp.body.into_async_read();
        tokio::io::copy(&mut stream, &mut file).await?;

        info!("Downloaded: {}", local_path);
    }
    Ok(())
}

pub async fn sync_r2_to_r2(
    client: &dyn R2ClientTrait,
    src: &R2Location,
    dest: &R2Location,
) -> Result<(), Box<dyn Error>> {
    let objects = client
        .list_objects(src.bucket.clone(), src.key_prefix.clone())
        .await?;

    for object in objects {
        let key = object.key().unwrap();
        let dest_key = if dest.key_prefix.is_empty() {
            key.to_string()
        } else {
            let s = format!("{}/{}", dest.key_prefix, key);
            s
        };

        let src_key = if src.key_prefix.is_empty() {
            key.to_string()
        } else {
            let s = format!("{}/{}", src.key_prefix, key);
            s
        };

        let resp = client
            .get_object(src.bucket.clone(), src_key.to_string())
            .await?;

        let body = resp.body.collect().await?.into_bytes().to_vec();

        client
            .put_object(dest.bucket.clone(), dest_key.clone(), body.clone())
            .await?;
        info!("Copied: {} to {}", key, dest_key);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_sdk_s3::primitives::ByteStream;
    use aws_sdk_s3::types::Object;
    use mockall::{mock, predicate::*};
    use std::error::Error;
    use tempfile::TempDir;
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;

    mock! {
        pub R2Client {}

        #[async_trait::async_trait]
        impl R2ClientTrait for R2Client {
            async fn put_object(
                &self,
                bucket: String,
                key: String,
                body: Vec<u8>,
            ) -> Result<(), Box<dyn Error>>;

            async fn get_object(
                &self,
                bucket: String,
                key: String,
            ) -> Result<ObjectOutput, Box<dyn Error>>;

            async fn list_objects(
                &self,
                bucket: String,
                prefix: String,
            ) -> Result<Vec<Object>, Box<dyn Error>>;
        }
    }

    #[tokio::test]
    async fn test_sync_local_to_r2() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let temp_dir_path = temp_dir.path();
        let file_path = temp_dir_path.join("file1.txt");

        let mut client = MockR2Client::new();

        client
            .expect_put_object()
            .with(
                eq("test-bucket".to_string()),
                eq("test-prefix/file1.txt".to_string()),
                eq(b"test data".to_vec()),
            )
            .returning(|_, _, _| Ok(()));

        let mut file = File::create(file_path).await.unwrap();
        file.write_all(b"test data").await.unwrap();
        drop(file);

        let result = sync_local_to_r2(
            &client,
            "test-bucket",
            temp_dir_path.to_str().unwrap(),
            "test-prefix",
            &None,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_sync_r2_to_local() {
        let mut client = MockR2Client::new();

        let mock_object = Object::builder().key("test-prefix/file1.txt").build();

        client
            .expect_list_objects()
            .with(eq("test-bucket".to_string()), eq("test-prefix".to_string()))
            .returning(move |_, _| Ok(vec![mock_object.clone()]));

        client
            .expect_get_object()
            .with(
                eq("test-bucket".to_string()),
                eq("test-prefix/file1.txt".to_string()),
            )
            .returning(|_, _| {
                Ok(ObjectOutput {
                    body: ByteStream::from_static(b"test data"),
                })
            });

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("file1.txt");

        let result = sync_r2_to_local(
            &client,
            "test-bucket",
            "test-prefix",
            temp_dir.path().to_str().unwrap(),
        )
        .await;

        assert!(result.is_ok());

        let mut downloaded_file = File::open(file_path).await.unwrap();
        let mut buffer = Vec::new();
        downloaded_file.read_to_end(&mut buffer).await.unwrap();
        assert_eq!(buffer, b"test data");
    }

    #[tokio::test]
    async fn test_sync_r2_to_r2() {
        let mut client = MockR2Client::new();

        let mock_object = Object::builder().key("file1.txt").build();

        client
            .expect_list_objects()
            .with(eq("src-bucket".to_string()), eq("test-prefix".to_string()))
            .returning(move |_, _| Ok(vec![mock_object.clone()]));

        client
            .expect_get_object()
            .with(
                eq("src-bucket".to_string()),
                eq("test-prefix/file1.txt".to_string()),
            )
            .returning(|_, _| {
                Ok(ObjectOutput {
                    body: ByteStream::from_static(b"test data"),
                })
            });

        client
            .expect_put_object()
            .with(
                eq("dest-bucket".to_string()),
                eq("dest-prefix/file1.txt".to_string()),
                eq(b"test data".to_vec()),
            )
            .returning(|_, _, _| Ok(()));

        let src_location = R2Location {
            bucket: "src-bucket".to_string(),
            key_prefix: "test-prefix".to_string(),
        };

        let dest_location = R2Location {
            bucket: "dest-bucket".to_string(),
            key_prefix: "dest-prefix".to_string(),
        };

        let result = sync_r2_to_r2(&client, &src_location, &dest_location).await;

        assert!(result.is_ok());
    }
}
