use crate::validator::ValidatorContainer;
use crate::DockerClient;
use testcontainers::core::{ExecCommand, WaitFor};
use testcontainers::{Container, GenericImage, RunnableImage};

pub struct Sandbox<'a> {
    pub container: Container<'a, GenericImage>,
    pub rpc_address: String,
}

impl<'a> Sandbox<'a> {
    pub const CONTAINER_RPC_PORT: u16 = 3000;
    pub const CONTAINER_NETWORK_PORT: u16 = 3001;

    pub async fn run(
        docker_client: &'a DockerClient,
        network: &str,
    ) -> anyhow::Result<Sandbox<'a>> {
        tracing::info!(network, "starting sandbox container");
        // TODO: combine macos and x86 tags under the same image
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        let image = GenericImage::new("ghcr.io/near/sandbox", "latest-aarch64")
            .with_wait_for(WaitFor::Nothing)
            .with_exposed_port(Self::CONTAINER_RPC_PORT);
        #[cfg(target_arch = "x86_64")]
        let image = GenericImage::new("ghcr.io/near/sandbox", "latest")
            .with_wait_for(WaitFor::Nothing)
            .with_exposed_port(Self::CONTAINER_RPC_PORT);
        let image: RunnableImage<GenericImage> = (
            image,
            vec![
                "--rpc-addr".to_string(),
                format!("0.0.0.0:{}", Self::CONTAINER_RPC_PORT),
                "--network-addr".to_string(),
                format!("0.0.0.0:{}", Self::CONTAINER_NETWORK_PORT),
            ],
        )
            .into();
        let image = image.with_network(network);
        let container = docker_client.cli.run(image);
        container.exec(ExecCommand {
            cmd: format!(
                "bash -c 'while [[ \"$(curl -H \"Content-type: application/json\" -X POST -s -o /dev/null -w ''%{{http_code}}'' -d ''{{
                \"jsonrpc\": \"2.0\",
                \"id\": \"dontcare\",
                \"method\": \"status\",
                \"params\": []
              }}'' localhost:{})\" != \"200\" ]]; do sleep 1; done; echo \"sandbox is ready to accept connections\"'",
                Self::CONTAINER_RPC_PORT
            ),
            ready_conditions: vec![WaitFor::StdErrMessage { message: "ready".to_string() }]
        });

        let ip_address = docker_client
            .get_network_ip_address(&container, network)
            .await?;
        let rpc_address = format!("http://{}:{}", ip_address, Self::CONTAINER_RPC_PORT);
        tracing::info!(rpc_address, "sandbox container is running");
        Ok(Sandbox {
            container,
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

impl<'a> ValidatorContainer<'a> for Sandbox<'a> {
    fn validator_container(&self) -> &Container<'a, GenericImage> {
        &self.container
    }
}
