use crate::DockerClient;
use testcontainers::core::WaitFor;
use testcontainers::{Container, GenericImage, RunnableImage};

pub struct Postgres<'a> {
    pub container: Container<'a, GenericImage>,
    pub connection_string: String,
}

impl<'a> Postgres<'a> {
    const POSTGRES_PORT: u16 = 5432;

    pub async fn run(
        docker_client: &'a DockerClient,
        network: &str,
    ) -> anyhow::Result<Postgres<'a>> {
        tracing::info!(network, "starting Postgres container");

        let image = GenericImage::new("darunrs/queryapi", "postgres")
            .with_env_var("POSTGRES_USER", "postgres")
            .with_env_var("POSTGRES_PASSWORD", "postgres")
            .with_exposed_port(Self::POSTGRES_PORT)
            .with_wait_for(WaitFor::message_on_stdout("ready to accept connections"));

        let image: RunnableImage<GenericImage> = image.into();
        let image = image.with_network(network);
        let container = docker_client.cli.run(image);

        let ip_address = docker_client
            .get_network_ip_address(&container, network)
            .await?;
        let connection_string = format!(
            "postgres://postgres:postgres@{}:{}/postgres",
            ip_address, Self::POSTGRES_PORT
        );

        tracing::info!(connection_string, "Postgres container is running",);

        Ok(Postgres {
            container,
            connection_string,
        })
    }

    pub fn host_postgres_address_ipv4(&self) -> String {
        let host_port = self.container.get_host_port_ipv4(Self::POSTGRES_PORT);
        format!("http://127.0.0.1:{host_port}")
    }
}