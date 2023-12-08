use near_workspaces::network::Sandbox;
use near_workspaces::types::{NearToken, SecretKey};
use near_workspaces::{AccessKey, Account, Contract, Worker};

pub struct NearcoreCtx {
    pub(crate) worker: Worker<Sandbox>,
}

impl NearcoreCtx {
    async fn initialize_social_db(worker: &Worker<Sandbox>) -> anyhow::Result<Contract> {
        let _span = tracing::info_span!("initializing socialdb contract");
        let social_db = worker
            .import_contract(&"social.near".parse()?, &near_workspaces::mainnet().await?)
            .transact()
            .await?;
        social_db
            .call("new")
            .max_gas()
            .transact()
            .await?
            .into_result()?;

        Ok(social_db)
    }

    // Linkdrop contains top-level account creation logic
    async fn initialize_linkdrop(worker: &Worker<Sandbox>) -> anyhow::Result<()> {
        let _span = tracing::info_span!("initializing linkdrop contract");
        let near_root_account = worker.root_account()?;
        worker
            .import_contract(&"near".parse()?, &near_workspaces::mainnet().await?)
            .dest_account_id(near_root_account.id())
            .transact()
            .await?;
        near_root_account
            .call(near_root_account.id(), "new")
            .max_gas()
            .transact()
            .await?
            .into_result()?;

        Ok(())
    }

    pub async fn new(worker: &Worker<Sandbox>) -> anyhow::Result<NearcoreCtx> {
        // Self::initialize_linkdrop(worker).await?;
        // TODO: move out of nearcore trait into its own ctx
        // let social_db = Self::initialize_social_db(worker).await?;

        Ok(NearcoreCtx {
            worker: worker.clone(),
        })
    }

    pub async fn create_account(
        &self,
        prefix: &str,
        initial_balance: NearToken,
    ) -> anyhow::Result<Account> {
        let _span = tracing::info_span!("creating account with random account id");
        let new_account = self
            .worker
            .root_account()?
            .create_subaccount(prefix)
            .initial_balance(initial_balance)
            .transact()
            .await?
            .into_result()?;

        tracing::info!(id = %new_account.id(), "account created");
        Ok(new_account)
    }

    pub async fn gen_rotating_keys(
        &self,
        account: &Account,
        amount: usize,
    ) -> anyhow::Result<Vec<SecretKey>> {
        let mut keys = Vec::with_capacity(amount + 1);
        keys.push(account.secret_key().clone());

        // Each batch transaction has a limit of BATCH_COUNT_LIMIT actions.
        const BATCH_COUNT_LIMIT: usize = 100;
        let num_batches = amount / BATCH_COUNT_LIMIT + 1;
        let rem_batches = amount % BATCH_COUNT_LIMIT;
        let batch_counts = (0..num_batches).map(|i| {
            if i == num_batches - 1 {
                rem_batches
            } else {
                BATCH_COUNT_LIMIT
            }
        });

        for batch_count in batch_counts {
            let mut batch_tx = account.batch(account.id());
            for _ in 0..batch_count {
                let sk = SecretKey::from_seed(
                    near_workspaces::types::KeyType::ED25519,
                    &rand::Rng::sample_iter(rand::thread_rng(), &rand::distributions::Alphanumeric)
                        .take(10)
                        .map(char::from)
                        .collect::<String>(),
                );
                batch_tx = batch_tx.add_key(sk.public_key(), AccessKey::full_access());
                keys.push(sk);
            }
            batch_tx.transact().await?.into_result()?;
        }

        Ok(keys)
    }

    /// Get the address the context is using to connect to the RPC of the network.
    pub fn rpc_address(&self) -> String {
        self.worker.rpc_addr()
    }
}
