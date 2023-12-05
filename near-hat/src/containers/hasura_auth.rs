use crate::validator::ValidatorContainer;
use crate::DockerClient;
use testcontainers::core::WaitFor;
use testcontainers::{Container, GenericImage, RunnableImage};

pub struct HasuraAuth<'a> {
    pub container: Container<'a, GenericImage>,
    pub auth_address: String,
}

impl<'a> HasuraAuth<'a> {
    pub const CONTAINER_HASURA_AUTH_PORT: u16 = 4000;

    pub async fn run(
        docker_client: &'a DockerClient,
        network: &str
    ) -> anyhow::Result<HasuraAuth<'a>> {
        tracing::info!("starting Hasura Auth container");

        let image = GenericImage::new("darunrs/queryapi", "hasura_auth")
            .with_env_var("PORT", Self::CONTAINER_HASURA_AUTH_PORT.to_string())
            .with_env_var("DEFAULT_HASURA_ROLE", "append")
            .with_wait_for(WaitFor::message_on_stderr("starting HTTP server on port 4000"))
            .with_exposed_port(Self::CONTAINER_HASURA_AUTH_PORT);
        let image: RunnableImage<GenericImage> = image.into();
        let image = image.with_network(network);
        let container = docker_client.cli.run(image);

        let ip_address = docker_client
            .get_network_ip_address(&container, network)
            .await?;
        let auth_address = format!("http://{}:{}", ip_address, Self::CONTAINER_HASURA_AUTH_PORT);

        tracing::info!(
            auth_address,
            "Hasura Auth container is running:"
        );

        Ok(HasuraAuth {
            container,
            auth_address,
        })
    }

    pub fn host_address_ipv4(&self) -> String {
        let host_port = self.container.get_host_port_ipv4(Self::CONTAINER_HASURA_AUTH_PORT);
        format!("http://127.0.0.1:{host_port}")
    }

    pub fn host_address_ipv6(&self) -> String {
        let host_port = self.container.get_host_port_ipv6(Self::CONTAINER_HASURA_AUTH_PORT);
        format!("http://[::1]:{host_port}")
    }
}

impl<'a> ValidatorContainer<'a> for HasuraAuth<'a> {
    fn validator_container(&self) -> &Container<'a, GenericImage> {
        &self.container
    }
}