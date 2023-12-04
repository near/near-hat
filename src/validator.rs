use crate::DockerClient;
use async_trait::async_trait;
use bollard::exec::{CreateExecOptions, StartExecResults};
use futures::StreamExt;
use near_crypto::KeyFile;
use testcontainers::{Container, GenericImage};

/// Container hosting a NEAR validator inside (e.g. Sandbox, Lake Indexer).
#[async_trait]
pub trait ValidatorContainer<'a> {
    fn validator_container(&self) -> &Container<'a, GenericImage>;

    async fn fetch(&self, docker_client: &DockerClient, path: &str) -> anyhow::Result<Vec<u8>> {
        tracing::info!(path, "fetching data from validator");
        let create_result = docker_client
            .docker
            .create_exec(
                self.validator_container().id(),
                CreateExecOptions::<&str> {
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    cmd: Some(vec!["cat", path]),
                    ..Default::default()
                },
            )
            .await?;

        let start_result = docker_client
            .docker
            .start_exec(&create_result.id, None)
            .await?;

        match start_result {
            StartExecResults::Attached { mut output, .. } => {
                let mut stream_contents = Vec::new();
                while let Some(chunk) = output.next().await {
                    stream_contents.extend_from_slice(&chunk?.into_bytes());
                }

                tracing::info!("data fetched");
                Ok(stream_contents)
            }
            StartExecResults::Detached => unreachable!("unexpected detached output"),
        }
    }

    async fn fetch_keys(&self, docker_client: &DockerClient) -> anyhow::Result<KeyFile> {
        let _span = tracing::info_span!("fetch_validator_keys");
        let key_data = self
            .fetch(docker_client, "/root/.near/validator_key.json")
            .await?;
        Ok(serde_json::from_slice(&key_data)?)
    }
}
