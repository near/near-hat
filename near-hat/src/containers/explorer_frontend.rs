use crate::DockerClient;
use testcontainers::core::WaitFor;
use testcontainers::{Container, GenericImage, RunnableImage};

pub struct ExplorerFrontend<'a> {
    pub container: Container<'a, GenericImage>,
}

impl<'a> ExplorerFrontend<'a> {
    pub const CONTAINER_PORT: u16 = 3000;

    pub async fn run(
        docker_client: &'a DockerClient,
        network: &str,
        backend_host_ip: &str,
        backend_host_port: u16,
        backend_internal_ip: &str,
        backend_internal_port: u16,
    ) -> anyhow::Result<ExplorerFrontend<'a>> {
        tracing::info!(network, "starting NEAR Explorer Frontend container");

        let image = GenericImage::new("morgsmccauley/explorer-frontend", "latest")
            .with_env_var("NEAR_EXPLORER_CONFIG__SEGMENT_WRITE_KEY", "7s4Na9mAfC7092R6pxrwpfBIAEek9Dne")
            .with_env_var(
                "NEAR_EXPLORER_CONFIG__NETWORKS",
                "{ \"localnet\": { \"explorerLink\": \"http://localhost:3000\", \"nearWalletProfilePrefix\": \"https://wallet.near.org/profile\" } }",
            )
            .with_env_var("NEAR_EXPLORER_CONFIG__BACKEND__HOSTS__MAINNET", "")
            .with_env_var(
                "NEAR_EXPLORER_CONFIG__BACKEND__HOSTS__LOCALNET",
                backend_host_ip,
            )
            .with_env_var(
                "NEAR_EXPLORER_CONFIG__BACKEND_SSR__HOSTS__LOCALNET",
                backend_internal_ip,
            )
            .with_env_var(
                "NEAR_EXPLORER_CONFIG__BACKEND__PORT",
                backend_host_port.to_string(),
            )
            .with_env_var(
                "NEAR_EXPLORER_CONFIG__BACKEND_SSR__PORT",
                backend_internal_port.to_string(),
            )
            .with_env_var("NEAR_EXPLORER_CONFIG__BACKEND__SECURE", "false")
            .with_wait_for(WaitFor::message_on_stdout("ready started server"))
            .with_exposed_port(Self::CONTAINER_PORT);

        let image: RunnableImage<GenericImage> = (image, vec![]).into();
        let image = image.with_network(network);
        let container = docker_client.cli.run(image);

        tracing::info!("NEAR Explorer Frontend container is running");

        Ok(ExplorerFrontend { container })
    }

    pub fn host_address_ipv4(&self) -> String {
        let host_port = self.container.get_host_port_ipv4(Self::CONTAINER_PORT);
        format!("http://127.0.0.1:{host_port}")
    }
}
