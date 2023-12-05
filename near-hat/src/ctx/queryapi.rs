use crate::client::DockerClient;
use crate::containers::hasura_auth::HasuraAuth;
use crate::containers::postgres::Postgres;
use crate::containers::hasura_graphql::HasuraGraphql;

pub struct QueryApiCtx<'a> {
    pub hasura_auth: HasuraAuth<'a>,
    pub postgres: Postgres<'a>,
    pub hasura_graphql: HasuraGraphql<'a>,
}

impl<'a> QueryApiCtx<'a> {
    pub async fn new(
        docker_client: &'a DockerClient,
        network: &str
    ) -> anyhow::Result<QueryApiCtx<'a>> {
        let hasura_auth = HasuraAuth::run(docker_client, network).await?;
        let postgres = Postgres::run(docker_client, network).await?;
        let hasura_graphql = HasuraGraphql::run(docker_client, network, &postgres.connection_string).await?;

        Ok(QueryApiCtx { 
            hasura_auth,
            postgres,
            hasura_graphql,
        })
    }
}