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

pub struct NearHat<'a> {
    pub queryapi_ctx: QueryApiCtx<'a>,
    pub lake_indexer_ctx: LakeIndexerCtx<'a>,
    pub nearcore_ctx: NearcoreCtx,
    pub relayer_ctx: RelayerCtx<'a>,
    pub explorer_ctx: ExplorerCtx<'a>,
}

impl<'a> NearHat<'a> {
    pub async fn new(
        docker_client: &'a DockerClient,
        network: &str,
    ) -> anyhow::Result<NearHat<'a>> {
        let lake_indexer_ctx = LakeIndexerCtx::new(docker_client, network).await?;
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
        )
        .await?;
        let explorer_ctx = ExplorerCtx::new(docker_client, network, &lake_indexer_ctx).await?;

        Ok(NearHat {
            queryapi_ctx,
            lake_indexer_ctx,
            nearcore_ctx,
            relayer_ctx,
            explorer_ctx,
        })
    }
}
