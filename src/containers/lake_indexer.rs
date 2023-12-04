use crate::DockerClient;
use testcontainers::core::WaitFor;
use testcontainers::{Container, GenericImage, RunnableImage};

pub struct LakeIndexer<'a> {
    pub container: Container<'a, GenericImage>,
    pub bucket_name: String,
    pub region: String,
    pub rpc_address: String,
}

impl<'a> LakeIndexer<'a> {
    pub const CONTAINER_RPC_PORT: u16 = 3030;

    pub async fn run(
        docker_client: &'a DockerClient,
        network: &str,
        s3_address: &str,
        bucket_name: String,
        region: String,
    ) -> anyhow::Result<LakeIndexer<'a>> {
        tracing::info!(
            network,
            s3_address,
            bucket_name,
            region,
            "starting NEAR Lake Indexer container"
        );

        let image = GenericImage::new(
            "ghcr.io/near/near-lake-indexer",
            "e6519c922435f3d18b5f2ddac5d1ec171ef4dd6b",
        )
        .with_env_var("AWS_ACCESS_KEY_ID", "FAKE_LOCALSTACK_KEY_ID")
        .with_env_var("AWS_SECRET_ACCESS_KEY", "FAKE_LOCALSTACK_ACCESS_KEY")
        .with_wait_for(WaitFor::message_on_stderr("Starting Streamer"))
        .with_exposed_port(Self::CONTAINER_RPC_PORT);
        let image: RunnableImage<GenericImage> = (
            image,
            vec![
                "--endpoint".to_string(),
                s3_address.to_string(),
                "--bucket".to_string(),
                bucket_name.clone(),
                "--region".to_string(),
                region.clone(),
                "--stream-while-syncing".to_string(),
                "sync-from-latest".to_string(),
            ],
        )
            .into();
        let image = image.with_network(network);
        let container = docker_client.cli.run(image);
        let ip_address = docker_client
            .get_network_ip_address(&container, network)
            .await?;
        let rpc_address = format!("http://{}:{}", ip_address, Self::CONTAINER_RPC_PORT);

        tracing::info!(
            bucket_name,
            region,
            rpc_address,
            "NEAR Lake Indexer container is running"
        );
        Ok(LakeIndexer {
            container,
            bucket_name,
            region,
            rpc_address,
        })
    }

    pub fn host_rpc_address_ipv4(&self) -> String {
        let host_port = self.container.get_host_port_ipv4(Self::CONTAINER_RPC_PORT);
        format!("http://127.0.0.1:{host_port}")
    }

    pub fn host_rpc_address_ipv6(&self) -> String {
        let host_port = self.container.get_host_port_ipv6(Self::CONTAINER_RPC_PORT);
        format!("http://[::1]:{host_port}")
    }
}
