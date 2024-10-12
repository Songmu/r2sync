mod r2_client;
mod sync;
mod utils;

use clap::Parser;
use log::info;
use r2_client::create_r2_client;
use std::error::Error;
use sync::{sync_local_to_r2, sync_r2_to_local, sync_r2_to_r2};
use utils::parse_r2_url;

#[derive(Parser)]
pub struct Cli {
    #[arg(short, long)]
    pub public_domain: Option<String>,
    #[arg(short, long)]
    pub dry_run: bool,
    pub source: String,
    pub destination: String,
}

impl Cli {
    pub fn parse_args() -> Result<Cli, Box<dyn Error>> {
        let args = Cli::parse();
        Ok(args)
    }

    pub async fn run(&self) -> Result<(), Box<dyn Error>> {
        if !self.source.starts_with("r2://") && !self.destination.starts_with("r2://") {
            return Err("One of the paths must start with r2://".into());
        }
        let client = create_r2_client().await?;

        if self.destination.starts_with("r2://") {
            let r2_dest = parse_r2_url(&self.destination)?;

            if !self.source.starts_with("r2://") {
                info!(
                    "Syncing from local directory {} to R2 bucket {}",
                    self.source, r2_dest.bucket
                );
                sync_local_to_r2(
                    &client,
                    &r2_dest.bucket,
                    &self.source,
                    &r2_dest.key_prefix,
                    &self.public_domain,
                )
                .await?;
            } else {
                let r2_src = parse_r2_url(&self.source)?;
                info!(
                    "Syncing from R2 bucket {} to R2 bucket {}",
                    r2_src.bucket, r2_dest.bucket
                );
                sync_r2_to_r2(&client, &r2_src, &r2_dest).await?;
            }
        } else {
            let r2_src = parse_r2_url(&self.source)?;
            info!(
                "Syncing from R2 bucket {} to local directory {}",
                r2_src.bucket, self.destination
            );
            sync_r2_to_local(
                &client,
                &r2_src.bucket,
                &r2_src.key_prefix,
                &self.destination,
            )
            .await?;
        }

        Ok(())
    }
}
