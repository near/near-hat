import { connect, Contract, keyStores, KeyPair } from 'near-api-js';
import { readFileSync } from 'fs';
import {restoreTestAccountKeys, getOrCreateAccount} from './testUtils.js'

describe('Hackathon Demo', () => {
    const connectionConfig = {
        networkId: "localnet",
        nodeUrl: "http://rpc.nearhat",
        walletUrl: "NONE",
        keyStore: new keyStores.InMemoryKeyStore()
    };

    test('Transaction events queryable after successful contract executions', async () => {
        const near = await connect(connectionConfig);
        const privateKeys = restoreTestAccountKeys(near);

        const queryApiAccount = await near.account("dev-queryapi.test.near");
        const indexerRegistry = new Contract(queryApiAccount, 'dev-queryapi.test.near', {
          viewMethods: ['read_indexer_function'],
          changeMethods: ['register_indexer_function'],
        });
        const code = readFileSync('data/indexer_code.js').toString();
        const schema = readFileSync('data/indexer_schema.sql').toString();
        await indexerRegistry.account.functionCall({
            contractId: indexerRegistry.contractId,
            methodName: 'register_indexer_function',
            args: {
                "function_name": "test_indexer",
                "code": code,
                "schema": schema,
                "filter_json": "{\"indexer_rule_kind\":\"Action\",\"matching_rule\":{\"rule\":\"ACTION_ANY\",\"affected_account_id\":\"*.near\",\"status\":\"SUCCESS\"}}"
            },
        });

        await new Promise(resolve => setTimeout(resolve, 2000));

        const multisafe = await getOrCreateAccount(near, "multisafe.near", 2);
        const usdtOwner = await getOrCreateAccount(near, "tether.multisafe.near", 1);
        const alice = await getOrCreateAccount(near, "alice.near", 1);
        const bob = await getOrCreateAccount(near, "bob.near", 1);
        
        const usdtContract = new Contract(usdtOwner, 'usdt.tether-token.near', {
          viewMethods: ['ft_balance_of', 'owner'],
          changeMethods: ['ft_transfer', 'mint', 'storage_deposit'],
        });
        await usdtContract.storage_deposit({
          account_id: alice.accountId
        }, "300000000000000", "10000000000000000000000");
        const response = await usdtContract.mint({
            account_id: alice.accountId,
            amount: "100000000"
        });
        // assert usdtContract.ft_balance_of({account_id: alice.accountId}).eq("100000000")
        // assert QueryAPI
        console.log(response);
    }, 30000);
});

function doLastElementsMatch(resultArray, expectedArray) {
    // Get the last elements of the result array based on the length of the expected array
    const lastElements = resultArray.slice(-expectedArray.length);

    // Compare the last elements with the expected array
    return lastElements.length === expectedArray.length &&
           lastElements.every((element, index) => element === expectedArray[index]);
}