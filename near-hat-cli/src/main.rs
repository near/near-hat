use clap::Parser;
use near_hat::{DockerClient, NearHat};
use near_primitives::types::AccountId;
use near_workspaces::network::Sandbox;
use near_workspaces::Worker;
use tokio::io::{stdin, AsyncReadExt};
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
pub enum Cli {
    Start {
        /// Contracts to spoon from mainnet.
        #[arg(long, value_parser, num_args = 1.., value_delimiter = ',')]
        contracts_to_spoon: Vec<AccountId>,
    },
}

async fn spoon_contracts(worker: &Worker<Sandbox>, contracts: &[AccountId]) -> anyhow::Result<()> {
    let _span = tracing::info_span!("spooning contracts");
    let readrpc_worker = near_workspaces::mainnet()
        .rpc_addr("https://beta.rpc.mainnet.near.org")
        .await?;
    for contract in contracts {
        worker
            .import_contract(contract, &readrpc_worker)
            .transact()
            .await?;
        tracing::info!(%contract, "imported contract");
        let state: Vec<u8> = readrpc_worker
            .view_state(contract)
            .await?
            .remove(b"STATE".as_slice())
            .unwrap();
        tracing::info!(%contract, state_size = state.len(), "pulled contract state");
        worker.patch_state(contract, b"STATE", &state).await?;
        tracing::info!(%contract, "patched contract state");
    }
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Install global collector configured based on RUST_LOG env var.
    let subscriber = tracing_subscriber::fmt()
        .with_thread_ids(true)
        .with_env_filter(EnvFilter::from_default_env());
    subscriber.init();

    match Cli::parse() {
        Cli::Start { contracts_to_spoon } => {
            let docker_client = DockerClient::default();
            let near_hat = NearHat::new(&docker_client, "nearhat").await?;
            spoon_contracts(&near_hat.lake_indexer_ctx.worker, &contracts_to_spoon).await?;
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
                "  Hasura Auth: {}",
                near_hat.queryapi_ctx.hasura_auth.host_address_ipv4()
            );
            println!(
                "  Postgres: {}",
                near_hat.queryapi_ctx.postgres.host_postgres_address_ipv4()
            );
            println!(
                "  Hasura Graphql: {}",
                near_hat.queryapi_ctx.hasura_graphql.host_address_ipv4()
            );
            println!(
                "  Redis: {}",
                near_hat.relayer_ctx.redis.host_redis_address_ipv4()
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
