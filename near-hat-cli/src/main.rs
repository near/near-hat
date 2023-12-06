use clap::Parser;
use near_hat::{DockerClient, NearHat};
use tokio::io::{stdin, AsyncReadExt};
use tracing_subscriber::EnvFilter;
use std::env;



#[derive(Parser, Debug)]
pub enum Cli {
    Start {},
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Install global collector configured based on RUST_LOG env var.
    let subscriber = tracing_subscriber::fmt()
        .with_thread_ids(true)
        .with_env_filter(EnvFilter::from_default_env());
    subscriber.init();

    match Cli::parse() {
        Cli::Start {} => {
            let docker_client = DockerClient::default();
            NearHat::new(&docker_client, "nearhat").await?;
            
            println!("\nNEARHat environment is ready with following gloval environment variables:");
            env::vars()
                .filter(|(key, _)| key.starts_with("NEARHAT"))
                .for_each(|(key, value)| {
                    println!("{}: {}", key, value);
                });

            println!("\nPress any button to exit and destroy all containers...");
            while stdin().read(&mut [0]).await? == 0 {
                tokio::time::sleep(std::time::Duration::from_millis(25)).await;
            }
        }
    }

    Ok(())
}
