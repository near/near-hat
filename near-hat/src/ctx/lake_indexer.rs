use std::cell::RefCell;
use std::fs::File;
use std::io::Write;
use std::rc::Rc;

use crate::client::DockerClient;
use crate::containers::lake_indexer::LakeIndexer;
use crate::containers::localstack::LocalStack;
use crate::validator::ValidatorContainer;
use near_workspaces::network::{Sandbox, ValidatorKey};
use near_workspaces::Worker;
use serde_json::{json, Value};

pub struct LakeIndexerCtx<'a> {
    pub localstack: LocalStack<'a>,
    pub lake_indexer: LakeIndexer<'a>,
    // FIXME: Technically this network is not sandbox, but workspaces does not support plain localnet
    pub worker: Worker<Sandbox>,
}

impl<'a> LakeIndexerCtx<'a> {
    pub async fn new(
        docker_client: &'a DockerClient,
        network: &str,
        key_json_ref: Rc<RefCell<Value>>,
    ) -> anyhow::Result<LakeIndexerCtx<'a>> {
        let s3_bucket = "localnet".to_string();
        let s3_region = "us-east-1".to_string();
        let localstack =
            LocalStack::run(docker_client, network, s3_bucket.clone(), s3_region.clone()).await?;

        let lake_indexer = LakeIndexer::run(
            docker_client,
            network,
            &localstack.s3_address,
            s3_bucket,
            s3_region,
        )
        .await?;

        let validator_key = lake_indexer.fetch_keys(docker_client).await?;

        tracing::info!("initializing sandbox worker");
        let worker = near_workspaces::sandbox()
            .rpc_addr(&lake_indexer.host_rpc_address_ipv4())
            .validator_key(ValidatorKey::Known(
                validator_key.account_id.to_string().parse()?,
                validator_key.secret_key.to_string().parse()?,
            ))
            .await?;

        key_json_ref.borrow_mut()[validator_key.account_id.to_string()] = json!(validator_key.secret_key.to_string());

        Ok(LakeIndexerCtx {
            localstack,
            lake_indexer,
            worker
        })
    }
}
