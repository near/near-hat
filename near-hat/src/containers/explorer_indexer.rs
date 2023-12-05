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
    ) -> anyhow::Result<ExplorerIndexer<'a>> {
        tracing::info!(network, "starting NEAR Lake Indexer container");

        let image = GenericImage::new(
            "us-central1-docker.pkg.dev/pagoda-data-stack-dev/cloud-run-source-deploy/indexer-explorer",
            "439ccdab3dfa60f503bf1ddcc4498595d5ad6339",
        )
        .with_env_var("AWS_ACCESS_KEY_ID", "FAKE_LOCALSTACK_KEY_ID")
        .with_env_var("AWS_SECRET_ACCESS_KEY", "FAKE_LOCALSTACK_ACCESS_KEY")
        .with_env_var("DATABASE_URL", "test")
        .with_wait_for(WaitFor::message_on_stderr("Starting Streamer"));

        let image: RunnableImage<GenericImage> = (
            image,
            vec!["mainnet".to_string(), "from-genesis".to_string()],
        )
            .into();
        let image = image.with_network(network);
        let container = docker_client.cli.run(image);

        tracing::info!("NEAR Explorer Indexer container is running");

        Ok(ExplorerIndexer { container })
    }
}
