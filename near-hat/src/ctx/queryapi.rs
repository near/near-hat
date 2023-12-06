use crate::client::DockerClient;
use crate::containers::coordinator::Coordinator;
use crate::containers::hasura_auth::HasuraAuth;
use crate::containers::postgres::Postgres;
use crate::containers::hasura_graphql::HasuraGraphql;
use std::fs::File;
use std::io::{Write, self};
use std::process::Command;
use std::path::Path;

pub struct QueryApiCtx<'a> {
    pub hasura_auth: HasuraAuth<'a>,
    pub postgres: Postgres<'a>,
    pub hasura_graphql: HasuraGraphql<'a>,
    pub coordinator: Coordinator<'a>,
}

impl<'a> QueryApiCtx<'a> {
    pub async fn new(
        docker_client: &'a DockerClient,
        network: &str,
        redis_address: &str,
        s3_address: &str,
        s3_bucket_name: &str,
        s3_region: &str,
        explorer_address: &str,
        rpc_address: &str,
    ) -> anyhow::Result<QueryApiCtx<'a>> {
        let hasura_auth = HasuraAuth::run(docker_client, network).await?;
        let postgres = Postgres::run(docker_client, network).await?;
        let hasura_graphql = HasuraGraphql::run(docker_client, network, &postgres.connection_string).await?;
        if let Err(e) = Self::update_config_and_deploy_hasura(&hasura_graphql.host_address_ipv4(), Path::new("./hasura")) {
            eprintln!("Failed to update config and run 'hasura deploy': {}", e);
        }
        let coordinator = Coordinator::run(
            docker_client, network, redis_address, 
            s3_address, s3_bucket_name, s3_region, 
            explorer_address, rpc_address).await?;

        Ok(QueryApiCtx { 
            hasura_auth,
            postgres,
            hasura_graphql,
            coordinator,
        })
    }

    fn update_config_and_deploy_hasura(endpoint: &str, hasura_folder: &Path) -> io::Result<()> {
        // Step 1: Update config.yaml
        let config_content = format!(
            "version: 3\nendpoint: {}\nadmin_secret: myadminsecretkey\nmetadata_directory: metadata\nactions:\n  kind: synchronous\n  handler_webhook_baseurl: http://localhost:3000",
            endpoint
        );
        
        let config_path = hasura_folder.join("config.yaml");
        let mut file = File::create(config_path)?;
        file.write_all(config_content.as_bytes())?;
    
        // Step 2: Run hasura deploy
        std::env::set_current_dir(hasura_folder)?;
    
        let output = Command::new("hasura")
            .arg("deploy")
            .output()?;
    
        if output.status.success() {
            println!("Hasura deploy executed successfully.");
        } else {
            eprintln!("Error in running Hasura deploy: {}", String::from_utf8_lossy(&output.stderr));
        }
    
        Ok(())
    }
}