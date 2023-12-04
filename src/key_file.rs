use anyhow::Context;
use near_primitives::types::AccountId;
use near_workspaces::types::SecretKey;
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;

pub struct KeyFile {
    file: File,
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
        let mut file = tempfile::tempfile_in(dir).context("creating temporary key file")?;
        file.write_all(&serde_json::to_vec(&key_file_json).context("serializing key file")?)
            .context("writing key file")?;
        Ok(Self { file })
    }
}
