mod client;
mod containers;
mod ctx;
mod validator;

pub use client::DockerClient;

use ctx::explorer::ExplorerCtx;
use ctx::lake_indexer::LakeIndexerCtx;
use ctx::nearcore::NearcoreCtx;
use ctx::queryapi::QueryApiCtx;
use ctx::relayer::RelayerCtx;
use std::process::{Command, Child};


pub struct NearHat<'a> {
    pub queryapi_ctx: QueryApiCtx<'a>,
    pub lake_indexer_ctx: LakeIndexerCtx<'a>,
    pub nearcore_ctx: NearcoreCtx,
    pub relayer_ctx: RelayerCtx<'a>,
    pub explorer_ctx: ExplorerCtx<'a>,
}

pub struct NearHatEnvironment<'a> {
    pub nearhat: NearHat<'a>,
    pub reverse_proxy_process: Child
}

impl<'a> NearHat<'a> {
    pub async fn new(
        docker_client: &'a DockerClient,
        network: &str,
    ) -> anyhow::Result<NearHatEnvironment<'a>> {
        let lake_indexer_ctx = LakeIndexerCtx::new(&docker_client, network).await?;
        let nearcore_ctx = NearcoreCtx::new(&lake_indexer_ctx.worker).await?;
        let relayer_ctx = RelayerCtx::new(docker_client, network, &nearcore_ctx).await?;
        let queryapi_ctx = QueryApiCtx::new(
            docker_client,
            network,
            &relayer_ctx.redis.redis_address,
            &lake_indexer_ctx.localstack.s3_address,
            &lake_indexer_ctx.localstack.s3_bucket,
            &lake_indexer_ctx.localstack.s3_region,
            &nearcore_ctx,
            &lake_indexer_ctx.lake_indexer.rpc_address,
            &lake_indexer_ctx.validator_key,
        )
        .await?;
        let explorer_ctx = ExplorerCtx::new(docker_client, network, &lake_indexer_ctx).await?;

        let nearhat = NearHat {
            queryapi_ctx,
            lake_indexer_ctx,
            nearcore_ctx,
            relayer_ctx,
            explorer_ctx,
        };

        let reverse_proxy_process = Self::start_reverse_proxy(&nearhat)?;

        Ok(NearHatEnvironment{
            nearhat,
            reverse_proxy_process
        })
    }

    fn start_reverse_proxy(nearhat: &NearHat<'_>) -> std::io::Result<Child> {
        let mut command = Command::new("mitmdump");

        command.arg("--mode").arg("regular").arg("-p").arg("80").arg("-s").arg("dns.py")
            .env("NEARHAT_RPC_PORT", &nearhat.lake_indexer_ctx.lake_indexer.host_rpc_port_ipv4().to_string())
            .env("NEARHAT_LAKE_S3_PORT", &nearhat.lake_indexer_ctx.localstack.host_port_ipv4().to_string())
            .env("NEARHAT_RELAYER_PORT", &nearhat.relayer_ctx.relayer.host_relayer_port_ipv4().to_string())
            .env("NEARHAT_EXPLORER_UI_PORT", &nearhat.explorer_ctx.frontend.host_frontend_port_ipv4().to_string())
            .stdout(std::process::Stdio::null());

        return command.spawn();
    }
}
