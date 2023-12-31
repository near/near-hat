use anyhow::anyhow;
use bollard::network::CreateNetworkOptions;
use bollard::service::Ipam;
use bollard::Docker;
use futures::lock::Mutex;
use once_cell::sync::Lazy;
use std::path::Path;
use testcontainers::clients::Cli;
use testcontainers::{Container, Image};

static NETWORK_MUTEX: Lazy<Mutex<i32>> = Lazy::new(|| Mutex::new(0));

pub struct DockerClient {
    pub(crate) docker: Docker,
    pub(crate) cli: Cli,
}

impl DockerClient {
    pub async fn get_network_ip_address<I: Image>(
        &self,
        container: &Container<'_, I>,
        network: &str,
    ) -> anyhow::Result<String> {
        let network_settings = self
            .docker
            .inspect_container(container.id(), None)
            .await?
            .network_settings
            .ok_or_else(|| anyhow!("missing NetworkSettings on container '{}'", container.id()))?;
        let ip_address = network_settings
            .networks
            .ok_or_else(|| {
                anyhow!(
                    "missing NetworkSettings.Networks on container '{}'",
                    container.id()
                )
            })?
            .get(network)
            .cloned()
            .ok_or_else(|| {
                anyhow!(
                    "container '{}' is not a part of network '{}'",
                    container.id(),
                    network
                )
            })?
            .ip_address
            .ok_or_else(|| {
                anyhow!(
                    "container '{}' belongs to network '{}', but is not assigned an IP address",
                    container.id(),
                    network
                )
            })?;

        Ok(ip_address)
    }

    pub async fn create_network(&self, network: &str) -> anyhow::Result<()> {
        let _lock = &NETWORK_MUTEX.lock().await;
        let list = self.docker.list_networks::<&str>(None).await?;
        if list.iter().any(|n| n.name == Some(network.to_string())) {
            return Ok(());
        }

        let create_network_options = CreateNetworkOptions {
            name: network,
            check_duplicate: true,
            driver: if cfg!(windows) {
                "transparent"
            } else {
                "bridge"
            },
            ipam: Ipam {
                config: None,
                ..Default::default()
            },
            ..Default::default()
        };
        let _response = &self.docker.create_network(create_network_options).await?;

        Ok(())
    }
}

impl Default for DockerClient {
    fn default() -> Self {
        let socket = std::env::var("DOCKER_HOST")
            .or(std::env::var("DOCKER_SOCK"))
            .unwrap_or_else(|_| {
                let socket = Path::new("/var/run/docker.sock");
                if socket.exists() {
                    "unix:///var/run/docker.sock".to_string()
                } else {
                    let home =
                        home::home_dir().expect("no home directory detected, please set HOME");
                    format!("unix://{}/.docker/run/docker.sock", home.display())
                }
            });
        Self {
            docker: Docker::connect_with_local(
                &socket,
                // 10 minutes timeout for all requests in case a lot of tests are being ran in parallel.
                600,
                bollard::API_DEFAULT_VERSION,
            )
            .unwrap(),
            cli: Default::default(),
        }
    }
}
