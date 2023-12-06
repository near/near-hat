use crate::DockerClient;
use testcontainers::core::WaitFor;
use testcontainers::{Container, GenericImage, RunnableImage};

pub struct ExplorerIndexer<'a> {
    pub container: Container<'a, GenericImage>,
}

impl<'a> ExplorerIndexer<'a> {
    pub async fn run(
        docker_client: &'a DockerClient,
        network: &str,
        s3_endpoint: &str,
        s3_bucket: &str,
        s3_region: &str,
    ) -> anyhow::Result<ExplorerIndexer<'a>> {
        tracing::info!(
            network,
            s3_endpoint,
            s3_bucket,
            s3_region,
            "starting NEAR Explorer Indexer container"
        );

        let image = GenericImage::new("explorer", "latest")
            .with_env_var("AWS_ACCESS_KEY_ID", "FAKE_LOCALSTACK_KEY_ID")
            .with_env_var("AWS_SECRET_ACCESS_KEY", "FAKE_LOCALSTACK_ACCESS_KEY")
            .with_env_var("DATABASE_URL", "test")
            .with_env_var("S3_REGION", s3_region)
            .with_env_var("AWS_REGION", s3_region)
            .with_env_var("S3_URL", s3_endpoint)
            .with_env_var("S3_BUCKET", s3_bucket)
            .with_env_var("RPC_URL", "not needed")
            .with_wait_for(WaitFor::message_on_stdout("Starting Indexer for Explorer"));

        let image: RunnableImage<GenericImage> = (
            image,
            vec![
                "localnet".to_string(),
                "from-block".to_string(),
                "0".to_string(),
            ],
        )
            .into();
        let image = image.with_network(network);
        let container = docker_client.cli.run(image);

        tracing::info!("NEAR Explorer Indexer container is running");

        Ok(ExplorerIndexer { container })
    }
}
