use clap::Parser;
use near_hat::{DockerClient, NearHat};
use near_primitives::types::AccountId;
use near_workspaces::network::Sandbox;
use near_workspaces::Worker;
use tokio::io::{stdin, AsyncReadExt};
use tracing_subscriber::EnvFilter;
extern crate ctrlc;

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
            .finality(near_workspaces::types::Finality::Final)
            .prefix(b"STATE".as_slice())
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
            let mut near_hat = NearHat::new(&docker_client, "nearhat").await?;
            spoon_contracts(
                &near_hat.nearhat.lake_indexer_ctx.worker,
                &contracts_to_spoon,
            )
            .await?;

            println!("\nNEARHat environment is ready:");
            println!(
                "  RPC: http://rpc.nearhat ({})",
                near_hat
                    .nearhat
                    .lake_indexer_ctx
                    .lake_indexer
                    .host_rpc_address_ipv4()
            );
            println!(
                "  Relayer: http://relayer.nearhat ({}), Creator Account: {}",
                near_hat
                    .nearhat
                    .relayer_ctx
                    .relayer
                    .host_http_address_ipv4(),
                near_hat.nearhat.relayer_ctx.creator_account.id()
            );
            println!(
                "  Relayer Redis: {}",
                near_hat.nearhat.relayer_ctx.redis.host_redis_address_ipv4()
            );
            println!(
                "  QueryAPI Hasura Auth: {}",
                near_hat
                    .nearhat
                    .queryapi_ctx
                    .hasura_auth
                    .host_address_ipv4()
            );
            println!(
                "  QueryAPI Postgres: {}",
                near_hat
                    .nearhat
                    .queryapi_ctx
                    .postgres
                    .host_postgres_address_ipv4()
            );
            println!(
                "  QueryAPI Graphql Playground: {}",
                near_hat
                    .nearhat
                    .queryapi_ctx
                    .hasura_graphql
                    .host_address_ipv4()
            );
            println!(
                "  Explorer Database: {}",
                near_hat
                    .nearhat
                    .explorer_ctx
                    .database
                    .host_postgres_connection_string()
            );
            println!(
                "  NEAR Lake S3: URL=http://lake.nearhat ({}), Region: {}, Bucket: {}",
                near_hat
                    .nearhat
                    .lake_indexer_ctx
                    .localstack
                    .host_s3_address_ipv4(),
                near_hat.nearhat.lake_indexer_ctx.localstack.s3_region,
                near_hat.nearhat.lake_indexer_ctx.localstack.s3_bucket
            );
            println!(
                "  Run `aws --endpoint-url=http://lake.nearhat s3 ls {}/000000000001/` to access block data",
                near_hat.nearhat.lake_indexer_ctx.localstack.s3_bucket
            );
            println!(
                "  Explorer Backend: {}",
                near_hat.nearhat.explorer_ctx.backend.host_address_ipv4()
            );
            println!(
                "  Explorer Frontend: http://explorer.nearhat ({})",
                near_hat.nearhat.explorer_ctx.frontend.host_address_ipv4()
            );

            println!("\nPress any button to exit and destroy all containers...");

            // Create a mutable flag to indicate if CTRL+C was received
            let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
            let r = running.clone();

            // Set up a CTRL+C handler
            ctrlc::set_handler(move || {
                r.store(false, std::sync::atomic::Ordering::SeqCst);
            }).expect("Error setting Ctrl-C handler");

            while stdin().read(&mut [0]).await? == 0 && running.load(std::sync::atomic::Ordering::SeqCst) {
                tokio::time::sleep(std::time::Duration::from_millis(25)).await;
            }
            println!("\nTerminating all Docker containers and reverse proxy...");
            let _ = near_hat.reverse_proxy_process.kill();
        }
    }

    Ok(())
}
