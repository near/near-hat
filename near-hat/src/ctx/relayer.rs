use super::nearcore::NearcoreCtx;
use crate::client::DockerClient;
use crate::containers::redis::Redis;
use crate::containers::relayer::Relayer;
use near_token::NearToken;
use near_workspaces::types::SecretKey;
use near_workspaces::Account;

pub struct RelayerCtx<'a> {
    pub redis: Redis<'a>,
    pub relayer: Relayer<'a>,
    pub creator_account: Account,
    pub creator_account_keys: Vec<SecretKey>,
}

impl<'a> RelayerCtx<'a> {
    pub async fn new(
        docker_client: &'a DockerClient,
        network: &str,
        nearcore_ctx: &NearcoreCtx,
    ) -> anyhow::Result<RelayerCtx<'a>> {
        let accounts_span = tracing::info_span!("initializing relayer accounts");
        let relayer_account = nearcore_ctx
            .create_account("relayer", NearToken::from_near(1000))
            .await?;
        let relayer_account_keys = nearcore_ctx.gen_rotating_keys(&relayer_account, 5).await?;

        let creator_account = nearcore_ctx
            .create_account("creator", NearToken::from_near(200))
            .await?;
        let creator_account_keys = nearcore_ctx.gen_rotating_keys(&creator_account, 5).await?;

        let social_account = nearcore_ctx
            .create_account("social", NearToken::from_near(1000))
            .await?;
        tracing::info!(
            relayer_account = %relayer_account.id(),
            creator_account = %creator_account.id(),
            social_account = %social_account.id(),
            "relayer accounts initialized",
        );
        drop(accounts_span);

        let redis = Redis::run(docker_client, network).await?;
        let relayer = Relayer::run(
            docker_client,
            network,
            &nearcore_ctx.rpc_address(),
            &redis.host_redis_address_ipv4(),
            relayer_account.id(),
            &relayer_account_keys,
            creator_account.id(),
            nearcore_ctx.social_db.id(),
            social_account.id(),
            social_account.secret_key(),
        )
        .await?;

        Ok(RelayerCtx::<'a> {
            redis,
            relayer,
            creator_account,
            creator_account_keys,
        })
    }
}
