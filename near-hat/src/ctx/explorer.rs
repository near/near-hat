use super::lake_indexer::LakeIndexerCtx;
use crate::client::DockerClient;
use crate::containers::explorer_backend::ExplorerBackend;
use crate::containers::explorer_database::ExplorerDatabase;
use crate::containers::explorer_frontend::ExplorerFrontend;
use crate::containers::explorer_indexer::ExplorerIndexer;

pub struct ExplorerCtx<'a> {
    pub indexer: ExplorerIndexer<'a>,
    pub database: ExplorerDatabase<'a>,
    pub backend: ExplorerBackend<'a>,
    pub frontend: ExplorerFrontend<'a>,
}

impl<'a> ExplorerCtx<'a> {
    pub async fn new(
        docker_client: &'a DockerClient,
        network: &str,
        lake_indexer_ctx: &LakeIndexerCtx<'a>,
    ) -> anyhow::Result<ExplorerCtx<'a>> {
        let database = ExplorerDatabase::run(docker_client, network).await?;

        let indexer = ExplorerIndexer::run(
            docker_client,
            network,
            &lake_indexer_ctx.localstack.s3_address,
            &lake_indexer_ctx.localstack.s3_bucket,
            &lake_indexer_ctx.localstack.s3_region,
            &database.connection_string,
        )
        .await?;

        let backend = ExplorerBackend::run(
            docker_client,
            network,
            &database.host,
            database.port,
            &lake_indexer_ctx.lake_indexer.rpc_address,
        )
        .await?;

        let frontend = ExplorerFrontend::run(
            docker_client,
            network,
            "127.0.0.1",
            backend.host_port_ipv4(),
            &backend.ip_address,
            backend.port,
        )
        .await?;

        Ok(ExplorerCtx {
            indexer,
            database,
            backend,
            frontend,
        })
    }
}
