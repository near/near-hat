use crate::DockerClient;
use testcontainers::{Container, GenericImage, RunnableImage};

pub struct Runner<'a> {
    pub container: Container<'a, GenericImage>,
    pub metrics_address: String,
}

impl<'a> Runner<'a> {
    const METRICS_PORT: u16 = 9180;

    pub async fn run(
        docker_client: &'a DockerClient,
        network: &str,
        region: &str,
        hasura_address: &str,
        redis_address: &str,
        postgres_host: &str,
        postgres_port: u16,
    ) -> anyhow::Result<Runner<'a>> {
        tracing::info!(network, "starting QueryAPI Runner container");

        let image = GenericImage::new("darunrs/queryapi", "runner")
            .with_env_var("AWS_ACCESS_KEY_ID", "FAKE_LOCALSTACK_KEY_ID")
            .with_env_var("AWS_SECRET_ACCESS_KEY", "FAKE_LOCALSTACK_ACCESS_KEY")
            .with_env_var("REGION", region)
            .with_env_var("REDIS_CONNECTION_STRING", redis_address)
            .with_env_var("HASURA_ENDPOINT", hasura_address)
            .with_env_var("HASURA_ADMIN_SECRET", "myadminsecretkey")
            .with_env_var("PORT", Self::METRICS_PORT.to_string())
            .with_env_var("PGHOST", postgres_host)
            .with_env_var("PGPORT", postgres_port.to_string())
            .with_env_var("PGUSER", "postgres")
            .with_env_var("PGPASSWORD", "postgres")
            .with_env_var("PGDATABASE", "postgres");
            // .with_wait_for(WaitFor::message_on_stdout("server running on http://localhost:9180"));

        let image: RunnableImage<GenericImage> = image.into();
        let image = image.with_network(network);
        let container = docker_client.cli.run(image);

        let ip_address = docker_client
            .get_network_ip_address(&container, network)
            .await?;
        let metrics_address = format!("http://{}:{}", ip_address, Self::METRICS_PORT);

        tracing::info!("QueryAPI Runner container is running",);

        Ok(Runner {
            container,
            metrics_address,
        })
    }
}
