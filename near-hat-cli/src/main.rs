use std::cell::RefCell;
use std::fs::File;
use std::io::Write;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use clap::Parser;
use near_hat::{DockerClient, NearHat};
use near_primitives::account::AccessKey;
use near_primitives::types::AccountId;
use near_workspaces::types::{PublicKey, KeyType};
use near_workspaces::{network::Sandbox, types::SecretKey};
use near_workspaces::{Worker, Contract};
use tokio::io::{stdin, AsyncReadExt};
use tracing_subscriber::EnvFilter;
use serde_json::{json, Value};
extern crate ctrlc;

#[derive(Parser, Debug)]
pub enum Cli {
    Start {
        /// Contracts to spoon from mainnet.
        #[arg(long, value_parser, num_args = 1.., value_delimiter = ',')]
        contracts_to_spoon: Vec<AccountId>,
    },
}

async fn patch_existing_account(worker: &Worker<Sandbox>, account_id: &AccountId, key_json_ref: Rc<RefCell<Value>>) -> anyhow::Result<()> {
    let _span = tracing::info_span!("creating account");
    let secret_key = near_workspaces::types::SecretKey::from_random(KeyType::ED25519);
    worker.patch(account_id).access_key(secret_key.public_key(), near_workspaces::AccessKey::full_access()).transact().await?;
    key_json_ref.borrow_mut()[account_id.to_string()] = json!(secret_key.to_string());
    tracing::info!(%account_id, "patched account");
    Ok(())
}

async fn spoon_contracts(worker: &Worker<Sandbox>, contracts: &[AccountId], key_json_ref: Rc<RefCell<Value>>) -> anyhow::Result<()> {
    patch_existing_account(worker, &AccountId::from_str("near").unwrap(), key_json_ref.clone()).await?;
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
        let state = readrpc_worker
            .view_state(contract)
            .finality(near_workspaces::types::Finality::Final)
            .prefix(b"STATE".as_slice())
            .await?
            .remove(b"STATE".as_slice())
            .unwrap();
        tracing::info!(%contract, state_size = state.len(), "pulled contract state");
        tracing::info!(%contract, "patched contract state");
        worker.patch_state(contract, b"STATE", &state).await?;
        patch_existing_account(worker, contract, key_json_ref.clone()).await?;
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
            let key_json_ref = Rc::new(RefCell::new(json!({})));
            let docker_client = DockerClient::default();
            let mut near_hat = NearHat::new(&docker_client, "nearhat", key_json_ref.clone()).await?;
            spoon_contracts(
                &near_hat.nearhat.lake_indexer_ctx.worker,
                &contracts_to_spoon,
                key_json_ref.clone()
            )
            .await?;

            let key_file_path = "tests/data/keys.json";
            let key_file = &mut File::create(key_file_path)?;
            let json_str = serde_json::to_string(&*key_json_ref.borrow())?;
            key_file.write_all(json_str.as_bytes())?;

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
                near_hat.nearhat.relayer_ctx.redis.host_redis_connection_ipv4()
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
                "  Graphql Playground: http://playground.nearhat ({}), password: {}",
                near_hat
                    .nearhat
                    .queryapi_ctx
                    .hasura_graphql
                    .host_address_ipv4(),
                near_hat
                    .nearhat
                    .queryapi_ctx
                    .hasura_graphql
                    .hasura_password()
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
