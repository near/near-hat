use std::cell::RefCell;
use std::fs;
use std::rc::Rc;

use near_token::NearToken;
use serde_json::{json, Value};

use crate::client::DockerClient;
use crate::containers::coordinator::Coordinator;
use crate::containers::hasura_auth::HasuraAuth;
use crate::containers::queryapi_postgres::QueryApiPostgres;
use crate::containers::hasura_graphql::HasuraGraphql;
use crate::containers::runner::Runner;

use super::nearcore::NearcoreCtx;

pub struct QueryApiCtx<'a> {
    pub hasura_auth: HasuraAuth<'a>,
    pub postgres: QueryApiPostgres<'a>,
    pub hasura_graphql: HasuraGraphql<'a>,
    pub coordinator: Coordinator<'a>,
    pub runner: Runner<'a>,
}

impl<'a> QueryApiCtx<'a> {
    pub async fn new(
        docker_client: &'a DockerClient,
        network: &str,
        redis_address: &str,
        s3_address: &str,
        s3_bucket_name: &str,
        s3_region: &str,
        nearcore: &NearcoreCtx,
        rpc_address: &str,
        key_json_ref: Rc<RefCell<Value>>,
    ) -> anyhow::Result<QueryApiCtx<'a>> {
        // Deploy registry contract and initialize it
        let wasm_bytes = fs::read("wasm/registry.wasm")?;
        let registry_holder = nearcore.create_account("dev-queryapi", NearToken::from_near(50)).await?;
        let registry_contract = registry_holder.deploy(&wasm_bytes).await?.unwrap();

        key_json_ref.borrow_mut()[registry_holder.id().to_string()] = json!(registry_holder.secret_key().to_string());

        // Set up dockers
        let hasura_auth = HasuraAuth::run(docker_client, network).await?;
        let postgres = QueryApiPostgres::run(docker_client, network).await?;
        let hasura_graphql = HasuraGraphql::run(docker_client, network, &hasura_auth.auth_address, &postgres.connection_string).await?;
        let coordinator = Coordinator::run(
            docker_client, 
            network, 
            redis_address, 
            s3_address, 
            s3_bucket_name, 
            s3_region, 
            rpc_address,
            registry_contract.id()).await?;

        let runner = Runner::run(
            docker_client, 
            network, 
            s3_region, 
            &hasura_graphql.hasura_address, 
            hasura_graphql.hasura_password().as_str(),
            redis_address, 
            &postgres.postgres_host, 
            postgres.postgres_port).await?;

        Ok(QueryApiCtx { 
            hasura_auth,
            postgres,
            hasura_graphql,
            coordinator,
            runner
        })
    }
}
