use crate::DockerClient;
use testcontainers::core::WaitFor;
use testcontainers::{Container, GenericImage, RunnableImage};

pub struct ExplorerDatabase<'a> {
    pub container: Container<'a, GenericImage>,
    pub connection_string: String,
    pub host: String,
    pub port: u16,
}

impl<'a> ExplorerDatabase<'a> {
    pub const CONTAINER_PORT: u16 = 5432;

    pub async fn run(
        docker_client: &'a DockerClient,
        network: &str,
    ) -> anyhow::Result<ExplorerDatabase<'a>> {
        tracing::info!(network, "starting NEAR Explorer Database container");

        let image = GenericImage::new("morgsmccauley/explorer-database", "latest")
            .with_wait_for(WaitFor::message_on_stdout(
                "database system is ready to accept connections",
            ))
            .with_exposed_port(Self::CONTAINER_PORT);

        let image: RunnableImage<GenericImage> = (image, vec![]).into();
        let image = image.with_network(network);
        let container = docker_client.cli.run(image);

        let ip_address = docker_client
            .get_network_ip_address(&container, network)
            .await?;

        let connection_string = format!(
            "postgres://postgres:postgres@{}:{}/postgres",
            ip_address,
            Self::CONTAINER_PORT
        );

        tracing::info!("NEAR Explorer Database container is running");

        Ok(ExplorerDatabase {
            container,
            connection_string,
            host: ip_address,
            port: Self::CONTAINER_PORT,
        })
    }

    pub fn host_postgres_connection_string(&self) -> String {
        let host_port = self.container.get_host_port_ipv4(Self::CONTAINER_PORT);
        format!("postgres://postgres:postgres@localhost:{host_port}/postgres")
    }

    pub fn host_postgres_port(&self) -> u16 {
        self.container.get_host_port_ipv4(Self::CONTAINER_PORT)
    }
}
