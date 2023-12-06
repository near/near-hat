use crate::DockerClient;
use bollard::exec::CreateExecOptions;
use testcontainers::core::WaitFor;
use testcontainers::{Container, GenericImage, RunnableImage};
use globalenv::set_var;

pub struct LocalStack<'a> {
    pub container: Container<'a, GenericImage>,
    pub s3_address: String,
    pub s3_bucket: String,
    pub s3_region: String,
}

impl<'a> LocalStack<'a> {
    const S3_CONTAINER_PORT: u16 = 4566;

    pub async fn run(
        docker_client: &'a DockerClient,
        network: &str,
        s3_bucket: String,
        s3_region: String,
    ) -> anyhow::Result<LocalStack<'a>> {
        tracing::info!(
            network,
            s3_bucket,
            s3_region,
            "Starting LocalStack container"
        );
        let image = GenericImage::new("localstack/localstack", "3.0.0")
            .with_wait_for(WaitFor::message_on_stdout("Running on"));
        let image: RunnableImage<GenericImage> = image.into();
        let image = image.with_network(network);
        let container = docker_client.cli.run(image);

        // Create the bucket
        let create_result = docker_client
            .docker
            .create_exec(
                container.id(),
                CreateExecOptions::<&str> {
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    cmd: Some(vec![
                        "awslocal",
                        "s3api",
                        "create-bucket",
                        "--bucket",
                        &s3_bucket,
                        "--region",
                        &s3_region,
                    ]),
                    ..Default::default()
                },
            )
            .await?;
        docker_client
            .docker
            .start_exec(&create_result.id, None)
            .await?;

        let ip_address = docker_client
            .get_network_ip_address(&container, network)
            .await?;
        let s3_address = format!("http://{}:{}", ip_address, Self::S3_CONTAINER_PORT);
        tracing::info!(s3_address, "LocalStack container is running");

        let result = LocalStack {
            container,
            s3_address,
            s3_bucket,
            s3_region,
        };

        set_var("NEARHAT_LAKE_S3", "http://lake.nearhat").unwrap();
        set_var("NEARHAT_LAKE_S3_LOCAL", result.host_s3_address_ipv4().as_str()).unwrap();
        set_var("NEARHAT_LAKE_S3_BUCKET", &result.s3_bucket.as_str()).unwrap();
        set_var("NEARHAT_LAKE_S3_REGION", &result.s3_region.as_str()).unwrap();

        Ok(result)
    }

    pub fn host_s3_address_ipv4(&self) -> String {
        let host_port = self.container.get_host_port_ipv4(Self::S3_CONTAINER_PORT);
        format!("http://127.0.0.1:{host_port}")
    }

    pub fn host_s3_address_ipv6(&self) -> String {
        let host_port = self.container.get_host_port_ipv6(Self::S3_CONTAINER_PORT);
        format!("http://[::1]:{host_port}")
    }
}
