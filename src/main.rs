mod cli;

use cli::Cli;
use env_logger::Env;
use log::error;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    match Cli::parse_args()?.run().await {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
