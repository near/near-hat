use clap::Parser;
use near_hat::{DockerClient, NearHat};
use tokio::io::{stdin, AsyncReadExt};
use tracing_subscriber::EnvFilter;
use std::env;
use globalenv::unset_var;


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
            let mut near_hat = NearHat::new(&docker_client, "nearhat").await?;

            println!("\nNEARHat environment is ready with following gloval environment variables:");
            env::vars()
                .filter(|(key, _)| key.starts_with("NEARHAT"))
                .for_each(|(key, value)| {
                    println!("{}: {}", key, value);
                });
            println!("\nNEARHat environment is ready:");
            println!(
                "  RPC: http://rpc.nearhat ({})",
                near_hat.nearhat
                    .lake_indexer_ctx
                    .lake_indexer
                    .host_rpc_address_ipv4()
            );
            println!(
                "  Explorer Database: {}",
                near_hat.nearhat
                    .explorer_indexer_ctx
                    .explorer_database
                    .host_postgres_connection_string()
            );
            println!(
                "  NEAR Lake S3: URL=http://lake.nearhat ({}), Region: {}, Bucket: {}",
                near_hat.nearhat.lake_indexer_ctx.localstack.host_s3_address_ipv4(),
                near_hat.nearhat.lake_indexer_ctx.localstack.s3_region,
                near_hat.nearhat.lake_indexer_ctx.localstack.s3_bucket
            );
            println!(
                "  Run `aws --endpoint-url=http://lake.nearhat s3 ls near-lake-custom/000000000001/` to access block data",
            );

            println!("\nPress any button to exit and destroy all containers...");
            while stdin().read(&mut [0]).await? == 0 {
                tokio::time::sleep(std::time::Duration::from_millis(25)).await;
            }

            env::vars()
                .filter(|(key, _)| key.starts_with("NEARHAT"))
                .for_each(|(key, _)| {
                    let _ = unset_var(&key);
                });
            let _ = near_hat.reverse_proxy_process.kill();
        }
    }

    Ok(())
}
