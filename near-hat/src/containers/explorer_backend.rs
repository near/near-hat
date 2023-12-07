use crate::DockerClient;
use testcontainers::core::WaitFor;
use testcontainers::{Container, GenericImage, RunnableImage};

pub struct ExplorerBackend<'a> {
    pub container: Container<'a, GenericImage>,
    pub ip_address: String,
    pub port: u16,
}

impl<'a> ExplorerBackend<'a> {
    pub const CONTAINER_PORT: u16 = 10000;

    pub async fn run(
        docker_client: &'a DockerClient,
        network: &str,
        database_host: &str,
        database_port: u16,
        rpc_url: &str,
    ) -> anyhow::Result<ExplorerBackend<'a>> {
        tracing::info!(network, "starting NEAR Explorer Backend container");

        let image = GenericImage::new("near-explorer-backend", "latest")
            .with_env_var("NEAR_EXPLORER_CONFIG__ARCHIVAL_RPC_URL", rpc_url)
            .with_env_var("NEAR_EXPLORER_CONFIG__NETWORK_NAME", "localnet")
            .with_env_var(
                "NEAR_EXPLORER_CONFIG__DB__READ_ONLY_INDEXER__HOST",
                database_host,
            )
            .with_env_var(
                "NEAR_EXPLORER_CONFIG__DB__READ_ONLY_INDEXER__DATABASE",
                "postgres",
            )
            .with_env_var(
                "NEAR_EXPLORER_CONFIG__DB__READ_ONLY_INDEXER__USER",
                "postgres",
            )
            .with_env_var(
                "NEAR_EXPLORER_CONFIG__DB__READ_ONLY_INDEXER__PASSWORD",
                "postgres",
            )
            .with_env_var(
                "NEAR_EXPLORER_CONFIG__DB__READ_ONLY_INDEXER__PORT",
                database_port.to_string(),
            )
            .with_env_var(
                "NEAR_EXPLORER_CONFIG__DB__READ_ONLY_ANALYTICS__HOST",
                "34.78.19.198",
            )
            .with_env_var(
                "NEAR_EXPLORER_CONFIG__DB__READ_ONLY_ANALYTICS__DATABASE",
                "indexer_analytics_mainnet",
            )
            .with_env_var(
                "NEAR_EXPLORER_CONFIG__DB__READ_ONLY_ANALYTICS__USER",
                "public_readonly",
            )
            .with_env_var(
                "NEAR_EXPLORER_CONFIG__DB__READ_ONLY_ANALYTICS__PASSWORD",
                "nearprotocol",
            )
            .with_env_var(
                "NEAR_EXPLORER_CONFIG__DB__READ_ONLY_TELEMETRY__HOST",
                "34.78.19.198",
            )
            .with_env_var(
                "NEAR_EXPLORER_CONFIG__DB__READ_ONLY_TELEMETRY__DATABASE",
                "telemetry_mainnet",
            )
            .with_env_var(
                "NEAR_EXPLORER_CONFIG__DB__READ_ONLY_TELEMETRY__USER",
                "public_readonly",
            )
            .with_env_var(
                "NEAR_EXPLORER_CONFIG__DB__READ_ONLY_TELEMETRY__PASSWORD",
                "nearprotocol",
            )
            .with_env_var("NEAR_EXPLORER_CONFIG__DB__WRITE_ONLY_TELEMETRY__HOST", "")
            .with_env_var(
                "NEAR_EXPLORER_CONFIG__DB__WRITE_ONLY_TELEMETRY__DATABASE",
                "telemetry_mainnet",
            )
            .with_env_var("NEAR_EXPLORER_CONFIG__DB__WRITE_ONLY_TELEMETRY__USER", "")
            .with_env_var(
                "NEAR_EXPLORER_CONFIG__DB__WRITE_ONLY_TELEMETRY__PASSWORD",
                "",
            )
            .with_wait_for(WaitFor::message_on_stdout("Explorer backend started"))
            .with_exposed_port(Self::CONTAINER_PORT);

        let image: RunnableImage<GenericImage> = (image, vec![]).into();
        let image = image.with_network(network);
        let container = docker_client.cli.run(image);

        let ip_address = docker_client
            .get_network_ip_address(&container, network)
            .await?;

        tracing::info!("NEAR Explorer Indexer container is running");

        Ok(ExplorerBackend {
            container,
            ip_address,
            port: Self::CONTAINER_PORT,
        })
    }

    pub fn host_address_ipv4(&self) -> String {
        let host_port = self.container.get_host_port_ipv4(Self::CONTAINER_PORT);
        format!("http://127.0.0.1:{host_port}")
    }

    pub fn host_port_ipv4(&self) -> u16 {
        self.container.get_host_port_ipv4(Self::CONTAINER_PORT)
    }
}
