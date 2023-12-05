mod client;
mod containers;
mod ctx;
mod validator;

pub use client::DockerClient;

use ctx::explorer_indexer::ExplorerIndexerCtx;
use ctx::lake_indexer::LakeIndexerCtx;
use ctx::nearcore::NearcoreCtx;
use ctx::relayer::RelayerCtx;
use ctx::queryapi::QueryApiCtx;

pub struct NearHat<'a> {
    pub queryapi_ctx: QueryApiCtx<'a>,
    pub lake_indexer_ctx: LakeIndexerCtx<'a>,
    pub nearcore_ctx: NearcoreCtx,
    pub relayer_ctx: RelayerCtx<'a>,
    pub explorer_indexer_ctx: ExplorerIndexerCtx<'a>,
}

impl<'a> NearHat<'a> {
    pub async fn new(
        docker_client: &'a DockerClient,
        network: &str,
    ) -> anyhow::Result<NearHat<'a>> {
        let queryapi_ctx = QueryApiCtx::new(docker_client, network).await?;
        let lake_indexer_ctx = LakeIndexerCtx::new(&docker_client, network).await?;
        let nearcore_ctx = NearcoreCtx::new(&lake_indexer_ctx.worker).await?;
        let relayer_ctx = RelayerCtx::new(&docker_client, network, &nearcore_ctx).await?;
        let explorer_indexer_ctx =
            ExplorerIndexerCtx::new(docker_client, network, &lake_indexer_ctx).await?;

        Ok(NearHat {
            queryapi_ctx,
            lake_indexer_ctx,
            nearcore_ctx,
            relayer_ctx,
            explorer_indexer_ctx,
        })
    }
}
