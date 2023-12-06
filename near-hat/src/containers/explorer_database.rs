use crate::DockerClient;
use testcontainers::core::WaitFor;
use testcontainers::{Container, GenericImage, RunnableImage};

pub struct ExplorerDatabase<'a> {
    pub container: Container<'a, GenericImage>,
    pub connection_string: String,
}

impl<'a> ExplorerDatabase<'a> {
    pub async fn run(
        docker_client: &'a DockerClient,
        network: &str,
    ) -> anyhow::Result<ExplorerDatabase<'a>> {
        tracing::info!(network, "starting NEAR Explorer Database container");

        let image = GenericImage::new("explorer-database", "latest").with_wait_for(
            WaitFor::message_on_stdout("database system is ready to accept connections"),
        );

        let image: RunnableImage<GenericImage> = (image, vec![]).into();
        let image = image.with_network(network);
        let container = docker_client.cli.run(image);

        let ip_address = docker_client
            .get_network_ip_address(&container, network)
            .await?;

        let connection_string = format!(
            "postgres://postgres:postgres@{}:{}/postgres",
            ip_address, 5432
        );

        tracing::info!("NEAR Explorer Database container is running");

        Ok(ExplorerDatabase {
            container,
            connection_string,
        })
    }
}
