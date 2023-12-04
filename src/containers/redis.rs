use crate::DockerClient;
use testcontainers::core::WaitFor;
use testcontainers::{Container, GenericImage, RunnableImage};

pub struct Redis<'a> {
    pub container: Container<'a, GenericImage>,
    pub redis_address: String,
}

impl<'a> Redis<'a> {
    // Port is hardcoded in the Redis image
    const CONTAINER_REDIS_PORT: u16 = 3000;

    pub async fn run(docker_client: &'a DockerClient, network: &str) -> anyhow::Result<Redis<'a>> {
        tracing::info!(network, "starting Redis container");
        let image = GenericImage::new("redis", "latest")
            .with_exposed_port(Self::CONTAINER_REDIS_PORT)
            .with_wait_for(WaitFor::message_on_stdout("Ready to accept connections"));
        let image: RunnableImage<GenericImage> = image.into();
        let image = image.with_network(network);
        let container = docker_client.cli.run(image);

        let ip_address = docker_client
            .get_network_ip_address(&container, network)
            .await?;
        let redis_address = format!("redis://{}:{}", ip_address, 6379);
        tracing::info!(redis_address, "Redis container is running",);
        Ok(Redis {
            container,
            redis_address,
        })
    }

    pub fn host_redis_address_ipv4(&self) -> String {
        let host_port = self
            .container
            .get_host_port_ipv4(Self::CONTAINER_REDIS_PORT);
        format!("http://127.0.0.1:{host_port}")
    }

    pub fn host_redis_address_ipv6(&self) -> String {
        let host_port = self
            .container
            .get_host_port_ipv6(Self::CONTAINER_REDIS_PORT);
        format!("http://[::1]:{host_port}")
    }
}
