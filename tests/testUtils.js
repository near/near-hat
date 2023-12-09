import { KeyPair, utils } from 'near-api-js';
import { readFileSync, writeFileSync } from 'fs';
import { Contract } from 'near-api-js';

export async function registerIndexer(indexerName, codePath, schemaPath, affectedAccountId, near) {
    const queryApiAccount = await near.account("dev-queryapi.test.near");
    const indexerRegistry = new Contract(queryApiAccount, 'dev-queryapi.test.near', {
        viewMethods: [],
        changeMethods: ['register_indexer_function'],
    });
    const code = readFileSync(codePath).toString();
    const schema = readFileSync(schemaPath).toString();

    await indexerRegistry.account.functionCall({
        contractId: indexerRegistry.contractId,
        methodName: "register_indexer_function",
        args: {
            "function_name": indexerName,
            "code": code,
            "schema": schema,
            "filter_json": `{\"indexer_rule_kind\":\"Action\",\"matching_rule\":{\"rule\":\"ACTION_ANY\",\"affected_account_id\":\"${affectedAccountId}\",\"status\":\"SUCCESS\"}}`
        },
    });
    await new Promise(resolve => setTimeout(resolve, 1000));
}

export function loadTestAccountKeys() {
    const privateKeysJson = readFileSync("data/keys.json", 'utf8');
    const privateKeys = JSON.parse(privateKeysJson);
    return privateKeys;
}

export function restoreTestAccountKeys(nearConnection) {
    const privateKeys = loadTestAccountKeys();
    Object.keys(privateKeys).forEach(key => {
      nearConnection.config.keyStore.setKey("localnet", key, KeyPair.fromString(privateKeys[key]));
    });
    return privateKeys;
}

export function saveTestAccountKey(accountId, privateKey) {
    const privateKeys = loadTestAccountKeys();
    privateKeys[accountId] = privateKey;
    writeFileSync("data/keys.json", JSON.stringify(privateKeys, null, 2), 'utf8');
}

export async function getOrCreateAccount(nearConnection, accountId, nearAmount) {
    const privateKeys = loadTestAccountKeys();
    const accountPK = privateKeys[accountId];
    if (accountPK) {
        const sk = KeyPair.fromString(accountPK);
        const account = await nearConnection.account(accountId);
        nearConnection.config.keyStore.setKey("localnet", accountId, accountPK);
        return account;
    }
    const accountCreatorId = accountId.split('.').slice(1).join('.');
    const sk = KeyPair.fromString(privateKeys[accountCreatorId]);
    const accountCreator = await nearConnection.account(accountCreatorId);
    await accountCreator.createAccount(accountId, sk.getPublicKey(), utils.format.parseNearAmount(nearAmount.toString()));
    const account = await nearConnection.account(accountId);
    nearConnection.config.keyStore.setKey("localnet", accountId, sk);
    saveTestAccountKey(accountId, privateKeys[accountCreatorId]);
    return account;
}

export async function fetchGraphQL(query, variables) {
    const response = await fetch('http://playground.nearhat/v1/graphql', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'x-hasura-admin-secret': 'nearhat'
      },
      body: JSON.stringify({
        query,
        variables
      })
    });
  
    return response.json();
  }