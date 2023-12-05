use crate::validator::ValidatorContainer;
use crate::DockerClient;
use testcontainers::core::WaitFor;
use testcontainers::{Container, GenericImage, RunnableImage};

pub struct HasuraGraphql<'a> {
    pub container: Container<'a, GenericImage>,
    pub graphql_address: String,
}

impl<'a> HasuraGraphql<'a> {
    pub const CONTAINER_HASURA_GRAPHQL_PORT: u16 = 8080;

    pub async fn run(
        docker_client: &'a DockerClient,
        network: &str,
        postgres_address: &str,
    ) -> anyhow::Result<HasuraGraphql<'a>> {
        tracing::info!("starting Hasura Graphql container");

        let image = GenericImage::new("hasura/graphql-engine", "latest")
            .with_env_var("HASURA_GRAPHQL_DATABASE_URL", postgres_address)
            .with_env_var("HASURA_GRAPHQL_ENABLE_CONSOLE", "true")
            .with_env_var("HASURA_GRAPHQL_DEV_MODE", "true")
            .with_env_var("HASURA_GRAPHQL_ENABLED_LOG_TYPES", "startup, http-log, webhook-log, websocket-log, query-log")
            .with_env_var("HASURA_GRAPHQL_ADMIN_SECRET", "myadminsecretkey")
            .with_env_var("HASURA_GRAPHQL_AUTH_HOOK", "http://hasura-auth:4000/auth")
            .with_exposed_port(Self::CONTAINER_HASURA_GRAPHQL_PORT);
        let image: RunnableImage<GenericImage> = image.into();
        let image = image.with_network(network);
        let container = docker_client.cli.run(image);

        let ip_address = docker_client
            .get_network_ip_address(&container, network)
            .await?;
        let graphql_address = format!("http://{}:{}", ip_address, Self::CONTAINER_HASURA_GRAPHQL_PORT);

        tracing::info!(
            graphql_address,
            "Hasura Graphql container is running:"
        );

        Ok(HasuraGraphql {
            container,
            graphql_address,
        })
    }

    pub fn host_address_ipv4(&self) -> String {
        let host_port = self.container.get_host_port_ipv4(Self::CONTAINER_HASURA_GRAPHQL_PORT);
        format!("http://127.0.0.1:{host_port}")
    }

    pub fn host_address_ipv6(&self) -> String {
        let host_port = self.container.get_host_port_ipv6(Self::CONTAINER_HASURA_GRAPHQL_PORT);
        format!("http://[::1]:{host_port}")
    }
}

impl<'a> ValidatorContainer<'a> for HasuraGraphql<'a> {
    fn validator_container(&self) -> &Container<'a, GenericImage> {
        &self.container
    }
}