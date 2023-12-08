use near_crypto::KeyFile;
use std::fs;

use near_token::NearToken;
use serde_json::json;

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
        validator_key: &KeyFile,
    ) -> anyhow::Result<QueryApiCtx<'a>> {
        // Deploy registry contract and initialize it
        let wasm_bytes = fs::read("wasm/registry.wasm")?;
        let registry_holder = nearcore.create_account("dev-queryapi", NearToken::from_near(50)).await?;
        let registry_contract = registry_holder.deploy(&wasm_bytes).await?.unwrap();

        // Write validator and queryapi account keys to file for use
        let keys = json!({
            validator_key.account_id.clone(): validator_key.secret_key,
            registry_holder.id().to_owned(): registry_holder.secret_key()
        });
        
        let keys_path = Path::new("./tests/data/keys.json");
        let mut file = File::create(&keys_path).expect("Unable to create file to store private keys");
        writeln!(file, "{}", keys.to_string()).expect("Unable to write private keys to file");

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
            redis_address, 
            &postgres.postgres_host, 
            postgres.postgres_port).await?;


        // Create generic indexer and register it
        // let register_function_args = serde_json::json!({
        //     "function_name": "test_sweat_blockheight",
        //     "code": "\n  const h = block.header().height;\n  await context.db.IndexerStorage.upsert({function_name: 'darunrs.near/test_sweat_blockheight', key_name: 'height', value: h.toString()}, [\"function_name\", \"key_name\"], [\"value\"]);\n",
        //     "schema": "CREATE TABLE\n  \"indexer_storage\" (\n    \"function_name\" TEXT NOT NULL,\n    \"key_name\" TEXT NOT NULL,\n    \"value\" TEXT NOT NULL,\n    PRIMARY KEY (\"function_name\", \"key_name\")\n  )\n",
        //     "filter_json": "{\"indexer_rule_kind\":\"Action\",\"matching_rule\":{\"rule\":\"ACTION_ANY\",\"affected_account_id\":\"*.near\",\"status\":\"SUCCESS\"}}"
        //   });
        // registry_holder.call(registry_holder.id(), "register_indexer_function").args(serde_json::to_vec(&register_function_args).unwrap()).transact().await?;

        Ok(QueryApiCtx { 
            hasura_auth,
            postgres,
            hasura_graphql,
            coordinator,
            runner
        })
    }
}
