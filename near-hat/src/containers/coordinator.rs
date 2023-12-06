use crate::DockerClient;
use testcontainers::core::WaitFor;
use testcontainers::{Container, GenericImage, RunnableImage};

pub struct Coordinator<'a> {
    pub container: Container<'a, GenericImage>,
    pub metrics_address: String,
}

impl<'a> Coordinator<'a> {
    const METRICS_PORT: u16 = 9180;

    pub async fn run(
        docker_client: &'a DockerClient,
        network: &str,
        redis_address: &str,
        s3_address: &str,
        s3_bucket_name: &str,
        s3_region: &str,
        explorer_address: &str,
        rpc_address: &str,
    ) -> anyhow::Result<Coordinator<'a>> {
        tracing::info!(network, "starting Coordinator container");

        let image = GenericImage::new("queryapi-mvp-coordinator", "latest")
            .with_env_var("AWS_ACCESS_KEY_ID", "FAKE_LOCALSTACK_KEY_ID")
            .with_env_var("AWS_SECRET_ACCESS_KEY", "FAKE_LOCALSTACK_ACCESS_KEY")
            .with_env_var("AWS_REGION", s3_region)
            .with_env_var("S3_URL", s3_address)
            .with_env_var("S3_BUCKET_NAME", s3_bucket_name)
            .with_env_var("RPC_ADDRESS", rpc_address)
            .with_env_var("EXPLORER_ADDRESS", explorer_address)
            .with_env_var("REDIS_CONNECTION_STRING", redis_address)
            .with_env_var("PORT", Self::METRICS_PORT.to_string())
            .with_env_var("REGISTRY_CONTRACT_ID", "dev-queryapi.dataplatform.near")
            .with_exposed_port(Self::METRICS_PORT);
            // .with_wait_for(WaitFor::message_on_stdout("Starting queryapi_coordinator..."));

        let image: RunnableImage<GenericImage> = (
            image,
            vec![
                "local-net".to_string(),
                "from-block".to_string(),
                "0".to_string(),
            ],
        ).into();
        let image = image.with_network(network);
        let container = docker_client.cli.run(image);

        let ip_address = docker_client
            .get_network_ip_address(&container, network)
            .await?;
        let metrics_address = format!("http://{}:{}", ip_address, Self::METRICS_PORT);

        tracing::info!(metrics_address, "Coordinator container is running",);

        Ok(Coordinator {
            container,
            metrics_address,
        })
    }
}