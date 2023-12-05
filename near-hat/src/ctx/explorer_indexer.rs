use crate::client::DockerClient;
use crate::containers::explorer_indexer::ExplorerIndexer;
use crate::containers::localstack::LocalStack;

pub struct ExplorerIndexerCtx<'a> {
    pub localstack: LocalStack<'a>,
    pub explorer_indexer: ExplorerIndexer<'a>,
}

impl<'a> ExplorerIndexerCtx<'a> {
    pub async fn new(
        docker_client: &'a DockerClient,
        network: &str,
    ) -> anyhow::Result<ExplorerIndexerCtx<'a>> {
        let s3_bucket = "near-lake-custom".to_string();
        let s3_region = "us-east-1".to_string();
        let localstack =
            LocalStack::run(docker_client, network, s3_bucket.clone(), s3_region.clone()).await?;

        let explorer_indexer = ExplorerIndexer::run(docker_client, network).await?;

        Ok(ExplorerIndexerCtx {
            localstack,
            explorer_indexer,
        })
    }
}
