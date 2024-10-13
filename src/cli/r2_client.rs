use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::config::Region;
use aws_sdk_s3::Client;
use aws_smithy_async::rt::sleep::TokioSleep;
use std::env;
use std::error::Error;
use std::sync::Arc;

pub async fn create_r2_client() -> Result<Client, Box<dyn Error>> {
    let account_id = env::var("R2_ACCOUNT_ID").expect("R2_ACCOUNT_ID environment variable not set");
    let endpoint_url = format!("https://{}.r2.cloudflarestorage.com", account_id);

    let access_key_id =
        env::var("R2_ACCESS_KEY_ID").expect("R2_ACCESS_KEY_ID environment variable not set");
    let secret_access_key = env::var("R2_SECRET_ACCESS_KEY")
        .expect("R2_SECRET_ACCESS_KEY environment variable not set");

    let credentials = Credentials::new(access_key_id, secret_access_key, None, None, "");

    let r2_config = aws_sdk_s3::config::Builder::new()
        .credentials_provider(credentials)
        .region(Region::new("auto"))
        .endpoint_url(endpoint_url)
        .sleep_impl(Arc::new(TokioSleep::new()))
        .build();

    Ok(Client::from_conf(r2_config))
}
