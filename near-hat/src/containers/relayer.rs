use crate::DockerClient;
use anyhow::Context;
use near_primitives::types::AccountId;
use near_workspaces::types::SecretKey;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tempfile::{NamedTempFile, TempDir};
use testcontainers::core::WaitFor;
use testcontainers::{Container, GenericImage, RunnableImage};
use toml::Value;

struct KeyFile {
    file: NamedTempFile,
}

impl KeyFile {
    pub fn temp_in_dir(
        account_id: &AccountId,
        account_sk: &SecretKey,
        dir: &TempDir,
    ) -> anyhow::Result<Self> {
        // Field names are following the relayer's configuration format. Do not change.
        let key_file_json = serde_json::json!({
            "account_id": account_id,
            "public_key": account_sk.public_key(),
            "private_key": account_sk,
        });
        let mut file = NamedTempFile::new_in(dir).context("creating temporary key file")?;
        file.write_all(&serde_json::to_vec(&key_file_json).context("serializing key file")?)
            .context("writing key file")?;
        Ok(Self { file })
    }
}

pub struct RelayerConfig {
    pub ip_address: [u8; 4],
    pub port: u16,
    pub relayer_account_id: AccountId,
    pub keys_filenames: Vec<PathBuf>,
    pub shared_storage_account_id: AccountId,
    pub shared_storage_keys_filename: String,
    pub whitelisted_contracts: Vec<AccountId>,
    pub whitelisted_delegate_action_receiver_ids: Vec<AccountId>,
    pub redis_url: String,
    pub social_db_contract_id: AccountId,
    pub rpc_url: String,
    pub wallet_url: String,
    pub explorer_transaction_url: String,
    pub rpc_api_key: String,
}

impl RelayerConfig {
    fn to_string(self) -> String {
        let mut config_table = Value::Table(toml::value::Table::new());
        let table = config_table.as_table_mut().unwrap();

        table.insert(
            "ip_address".to_string(),
            Value::Array(
                self.ip_address
                    .into_iter()
                    .map(|ip| Value::Integer(i64::from(ip)))
                    .collect(),
            ),
        );
        table.insert("port".to_string(), Value::Integer(i64::from(self.port)));

        table.insert(
            "relayer_account_id".to_string(),
            Value::String(self.relayer_account_id.to_string()),
        );
        table.insert(
            "keys_filenames".to_string(),
            Value::Array(
                self.keys_filenames
                    .into_iter()
                    .map(|filename| Value::String(filename.to_str().unwrap().to_string()))
                    .collect(),
            ),
        );

        table.insert(
            "shared_storage_account_id".to_string(),
            Value::String(self.shared_storage_account_id.to_string()),
        );
        table.insert(
            "shared_storage_keys_filename".to_string(),
            Value::String(self.shared_storage_keys_filename),
        );

        table.insert(
            "whitelisted_contracts".to_string(),
            Value::Array(
                self.whitelisted_contracts
                    .into_iter()
                    .map(|account_id| Value::String(account_id.to_string()))
                    .collect(),
            ),
        );
        table.insert(
            "whitelisted_delegate_action_receiver_ids".to_string(),
            Value::Array(
                self.whitelisted_delegate_action_receiver_ids
                    .into_iter()
                    .map(|account_id| Value::String(account_id.to_string()))
                    .collect(),
            ),
        );

        table.insert("redis_url".to_string(), Value::String(self.redis_url));
        table.insert(
            "social_db_contract_id".to_string(),
            Value::String(self.social_db_contract_id.to_string()),
        );

        table.insert("rpc_url".to_string(), Value::String(self.rpc_url));
        table.insert("wallet_url".to_string(), Value::String(self.wallet_url)); // not used
        table.insert(
            "explorer_transaction_url".to_string(),
            Value::String(self.explorer_transaction_url),
        ); // not used
        table.insert("rpc_api_key".to_string(), Value::String(self.rpc_api_key)); // not used

        toml::to_string(&config_table).expect("failed to serialize relayer config")
    }
}

pub struct Relayer<'a> {
    pub container: Container<'a, GenericImage>,
    pub http_address: String,
    // Keep key file handles to ensure that tmp files outlive the container.
    _social_account_key_file: KeyFile,
    _relayer_keyfiles: Vec<KeyFile>,
}

impl<'a> Relayer<'a> {
    pub const CONTAINER_PORT: u16 = 3000;

    pub async fn run(
        docker_client: &'a DockerClient,
        network: &str,
        near_rpc: &str,
        redis_url: &str,
        relayer_account_id: &AccountId,
        relayer_account_sks: &[near_workspaces::types::SecretKey],
        creator_account_id: &AccountId,
        social_db_contract_id: &AccountId,
        social_account_id: &AccountId,
        social_account_sk: &near_workspaces::types::SecretKey,
    ) -> anyhow::Result<Relayer<'a>> {
        tracing::info!(
            network,
            near_rpc,
            redis_url,
            %relayer_account_id,
            %creator_account_id,
            %social_db_contract_id,
            %social_account_id,
            "running relayer container"
        );

        // Create tmp folder to store relayer configs
        let relayer_config_dir =
            tempfile::tempdir().context("creating relayer config directory")?;

        // Create dir for keys
        let key_dir =
            tempfile::tempdir_in(&relayer_config_dir).context("creating relayer keys directory")?;

        // Create JSON key files
        let social_account_key_file =
            KeyFile::temp_in_dir(social_account_id, social_account_sk, &key_dir)?;
        let mut relayer_keyfiles = Vec::with_capacity(relayer_account_sks.len());
        for relayer_sk in relayer_account_sks {
            relayer_keyfiles.push(KeyFile::temp_in_dir(
                relayer_account_id,
                relayer_sk,
                &key_dir,
            )?);
        }

        // Create relayer config file
        let relayer_config = RelayerConfig {
            ip_address: [0, 0, 0, 0],
            port: Self::CONTAINER_PORT,
            relayer_account_id: relayer_account_id.clone(),
            keys_filenames: relayer_keyfiles
                .iter()
                .map(|kf| {
                    kf.file
                        .path()
                        .strip_prefix(&relayer_config_dir)
                        .unwrap()
                        .to_path_buf()
                })
                .collect(),
            shared_storage_account_id: social_account_id.clone(),
            shared_storage_keys_filename: format!("./account_keys/{}.json", social_account_id),
            whitelisted_contracts: vec![creator_account_id.clone()],
            whitelisted_delegate_action_receiver_ids: vec![creator_account_id.clone()],
            redis_url: redis_url.to_string(),
            social_db_contract_id: social_db_contract_id.clone(),
            rpc_url: near_rpc.to_string(),
            wallet_url: "https://wallet.testnet.near.org".to_string(),
            explorer_transaction_url: "https://explorer.testnet.near.org/transactions/".to_string(),
            rpc_api_key: "".to_string(),
        };
        let relayer_config_path = relayer_config_dir.path().join("config.toml");
        let mut relayer_config_file = File::create(&relayer_config_path).unwrap_or_else(|_| {
            panic!(
                "failed to create relayer config file at {}",
                relayer_config_path.display()
            )
        });
        relayer_config_file
            .write_all(relayer_config.to_string().as_bytes())
            .unwrap_or_else(|_| {
                panic!(
                    "failed to write relayer config to {}",
                    relayer_config_path.display()
                )
            });

        let image = GenericImage::new(
            "ghcr.io/near/os-relayer",
            "12ba6e35690df3979fce0b36a41d0ca0db9c0ab4",
        )
        .with_wait_for(WaitFor::message_on_stdout("listening on"))
        .with_exposed_port(Self::CONTAINER_PORT)
        .with_volume(
            relayer_config_path.to_str().unwrap(),
            "/relayer-app/config.toml",
        )
        .with_volume(
            key_dir.path().to_str().unwrap(),
            "/relayer-app/account_keys", // FIXME: directory name is probably going to be mangled, so it wouldn't work like that
        )
        .with_env_var("RUST_LOG", "DEBUG");

        let image: RunnableImage<GenericImage> = image.into();
        let image = image.with_network(network);
        let container = docker_client.cli.run(image);

        let ip_address = docker_client
            .get_network_ip_address(&container, network)
            .await?;
        let http_address = format!("http://{}:{}", ip_address, Self::CONTAINER_PORT);
        tracing::info!(http_address, "Relayer container is running");

        Ok(Relayer {
            container,
            http_address,
            _social_account_key_file: social_account_key_file,
            _relayer_keyfiles: relayer_keyfiles,
        })
    }

    pub fn host_relayer_port_ipv4(&self) -> u16 {
        return self.container.get_host_port_ipv4(Self::CONTAINER_PORT);
    }

    pub fn host_http_address_ipv4(&self) -> String {
        let host_port = self.container.get_host_port_ipv4(Self::CONTAINER_PORT);
        format!("http://127.0.0.1:{host_port}")
    }

    pub fn host_http_address_ipv6(&self) -> String {
        let host_port = self.container.get_host_port_ipv6(Self::CONTAINER_PORT);
        format!("http://[::1]:{host_port}")
    }
}
