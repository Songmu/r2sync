use super::utils::R2Location;
use aws_sdk_s3::model::ObjectCannedAcl;
use aws_sdk_s3::Client;
use log::info;
use reqwest::Client as ReqwestClient;
use std::error::Error;
use tokio::io::AsyncReadExt;

pub async fn sync_local_to_r2(
    client: &Client,
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
                        info!("File already exists: {}", key);
                        continue;
                    }
                }
            }

            client
                .put_object()
                .bucket(bucket_name)
                .key(key.clone())
                .body(buffer.into())
                .acl(ObjectCannedAcl::Private)
                .send()
                .await?;

            info!("Uploaded: {}", key);
        }
    }
    Ok(())
}

pub async fn sync_r2_to_local(
    client: &Client,
    bucket_name: &str,
    key_prefix: &str,
    local_dir: &str,
) -> Result<(), Box<dyn Error>> {
    let objects = client
        .list_objects_v2()
        .bucket(bucket_name)
        .prefix(key_prefix)
        .send()
        .await?;

    for object in objects.contents().unwrap_or_default() {
        let key = object.key().unwrap();
        let file_name = key.strip_prefix(key_prefix).unwrap_or(key);
        let local_path = format!("{}/{}", local_dir, file_name);

        let resp = client
            .get_object()
            .bucket(bucket_name)
            .key(key)
            .send()
            .await?;

        let mut file = tokio::fs::File::create(&local_path).await?;
        let mut stream = resp.body.into_async_read();
        tokio::io::copy(&mut stream, &mut file).await?;

        info!("Downloaded: {}", local_path);
    }
    Ok(())
}

pub async fn sync_r2_to_r2(
    client: &Client,
    src: &R2Location,
    dest: &R2Location,
) -> Result<(), Box<dyn Error>> {
    let objects = client
        .list_objects_v2()
        .bucket(&src.bucket)
        .prefix(&src.key_prefix)
        .send()
        .await?;

    for object in objects.contents().unwrap_or_default() {
        let key = object.key().unwrap();
        let dest_key = format!(
            "{}/{}",
            dest.key_prefix,
            key.strip_prefix(&src.key_prefix).unwrap_or(key)
        );

        let resp = client
            .get_object()
            .bucket(&src.bucket)
            .key(key)
            .send()
            .await?;

        let body = resp.body.collect().await?.into_bytes();

        client
            .put_object()
            .bucket(&dest.bucket)
            .key(dest_key.clone())
            .body(body.into())
            .acl(ObjectCannedAcl::Private)
            .send()
            .await?;

        info!("Copied: {} to {}", key, dest_key);
    }
    Ok(())
}
