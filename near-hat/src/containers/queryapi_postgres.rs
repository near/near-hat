use crate::DockerClient;
use testcontainers::core::WaitFor;
use testcontainers::{Container, GenericImage, RunnableImage};

pub struct QueryApiPostgres<'a> {
    pub container: Container<'a, GenericImage>,
    pub connection_string: String,
    pub postgres_host: String,
    pub postgres_port: u16,
}

impl<'a> QueryApiPostgres<'a> {
    const POSTGRES_PORT: u16 = 5432;
    const POSTGRES_USERNAME: &str = "postgres";
    const POSTGRES_PASSWORD: &str = "postgres";


    pub async fn run(
        docker_client: &'a DockerClient,
        network: &str,
    ) -> anyhow::Result<QueryApiPostgres<'a>> {
        tracing::info!(network, "starting Postgres container");

        let image = GenericImage::new("darunrs/queryapi", "postgres")
            .with_env_var("POSTGRES_USER", Self::POSTGRES_USERNAME)
            .with_env_var("POSTGRES_PASSWORD", Self::POSTGRES_PASSWORD)
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

        Ok(QueryApiPostgres {
            container,
            connection_string,
            postgres_host: ip_address,
            postgres_port: Self::POSTGRES_PORT,
        })
    }

    pub fn host_postgres_address_ipv4(&self) -> String {
        let host_port = self.container.get_host_port_ipv4(Self::POSTGRES_PORT);
        format!("http://127.0.0.1:{host_port}")
    }
}