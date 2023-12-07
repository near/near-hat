use super::lake_indexer::LakeIndexerCtx;
use crate::client::DockerClient;
use crate::containers::explorer_database::ExplorerDatabase;
use crate::containers::explorer_indexer::ExplorerIndexer;

pub struct ExplorerIndexerCtx<'a> {
    pub explorer_indexer: ExplorerIndexer<'a>,
    pub explorer_database: ExplorerDatabase<'a>,
}

impl<'a> ExplorerIndexerCtx<'a> {
    pub async fn new(
        docker_client: &'a DockerClient,
        network: &str,
        lake_indexer_ctx: &LakeIndexerCtx<'a>,
    ) -> anyhow::Result<ExplorerIndexerCtx<'a>> {
        let explorer_database = ExplorerDatabase::run(docker_client, network).await?;

        let explorer_indexer = ExplorerIndexer::run(
            docker_client,
            network,
            &lake_indexer_ctx.localstack.s3_address,
            &lake_indexer_ctx.localstack.s3_bucket,
            &lake_indexer_ctx.localstack.s3_region,
            &explorer_database.connection_string,
        )
        .await?;

        Ok(ExplorerIndexerCtx {
            explorer_indexer,
            explorer_database,
        })
    }
}