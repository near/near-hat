use crate::client::DockerClient;
use crate::containers::hasura_auth::HasuraAuth;

pub struct QueryApiCtx<'a> {
    pub hasura_auth: HasuraAuth<'a>,
}

impl<'a> QueryApiCtx<'a> {
    pub async fn new(
        docker_client: &'a DockerClient,
        network: &str
    ) -> anyhow::Result<QueryApiCtx<'a>> {
        let hasura_auth = HasuraAuth::run(docker_client, network).await?;

        Ok(QueryApiCtx { hasura_auth })
    }
}