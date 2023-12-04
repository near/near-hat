mod client;
mod containers;
mod ctx;
mod validator;

use client::DockerClient;
use ctx::lake_indexer::LakeIndexerCtx;
use ctx::nearcore::NearcoreCtx;
use ctx::relayer::RelayerCtx;

pub struct NearHat<'a> {
    pub lake_indexer_ctx: LakeIndexerCtx<'a>,
    pub nearcore_ctx: NearcoreCtx,
    pub relayer_ctx: RelayerCtx<'a>,
}

impl<'a> NearHat<'a> {
    pub async fn new(
        docker_client: &'a DockerClient,
        network: &str,
    ) -> anyhow::Result<NearHat<'a>> {
        let lake_indexer_ctx = LakeIndexerCtx::new(&docker_client, network).await?;
        let nearcore_ctx = NearcoreCtx::new(&lake_indexer_ctx.worker).await?;
        let relayer_ctx = RelayerCtx::new(&docker_client, network, &nearcore_ctx).await?;

        Ok(NearHat {
            lake_indexer_ctx,
            nearcore_ctx,
            relayer_ctx,
        })
    }
}
