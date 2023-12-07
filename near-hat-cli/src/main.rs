use clap::Parser;
use near_hat::{DockerClient, NearHat};
use tokio::io::{stdin, AsyncReadExt};
use tracing_subscriber::EnvFilter;

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
            let near_hat = NearHat::new(&docker_client, "nearhat").await?;
            println!("\nNEARHat environment is ready:");
            println!(
                "  RPC: {}",
                near_hat
                    .lake_indexer_ctx
                    .lake_indexer
                    .host_rpc_address_ipv4()
            );
            println!(
                "  Lake Indexer S3 URL: {}",
                near_hat.lake_indexer_ctx.localstack.host_s3_address_ipv4()
            );
            println!(
                "  Lake Indexer S3 Region: {}",
                near_hat.lake_indexer_ctx.localstack.s3_region
            );
            println!(
                "  Lake Indexer S3 Bucket: {}",
                near_hat.lake_indexer_ctx.localstack.s3_bucket
            );
            println!(
                "  Explorer Database: {}",
                near_hat
                    .explorer_indexer_ctx
                    .explorer_database
                    .host_postgres_connection_string()
            );

            println!("\nPress any button to exit and destroy all containers...");
            while stdin().read(&mut [0]).await? == 0 {
                tokio::time::sleep(std::time::Duration::from_millis(25)).await;
            }
        }
    }

    Ok(())
}
