import { connect, Contract, keyStores, KeyPair } from 'near-api-js';
import { readFileSync } from 'fs';
import { fullAccessKey } from 'near-api-js/lib/transaction';
import assert from 'assert';

describe('Hackathon Demo', () => {
    const connectionConfig = {
        networkId: "localnet",
        nodeUrl: "http://rpc.nearhat",
        walletUrl: "NONE",
        keyStore: new keyStores.InMemoryKeyStore()
    };
    const privateKeysJson = readFileSync("data/keys.json", 'utf8');
    const privateKeys = JSON.parse(privateKeysJson);
    Object.keys(privateKeys).forEach(key => {
        connectionConfig.keyStore.setKey("localnet", key, KeyPair.fromString(privateKeys[key]));
    });

    test('Transaction events queryable after successful contract executions', async () => {
        const nearConnection = await connect(connectionConfig);
        // const base_account = await nearConnection.account("dev-queryapi.test.near");
        // const registry_contract = new Contract(base_account, 'dev-queryapi.test.near', {
        //     viewMethods: ['read_indexer_function'],
        //     changeMethods: ['register_indexer_function'],
        // });
        // const code = readFileSync('data/indexer_code.js').toString();
        // const schema = readFileSync('data/indexer_schema.sql').toString();
        // await registry_contract.account.functionCall({
        //     contractId: registry_contract.contractId,
        //     methodName: 'register_indexer_function',
        //     args: {
        //         "function_name": "test_indexer",
        //         "code": code,
        //         "schema": schema,
        //         "filter_json": "{\"indexer_rule_kind\":\"Action\",\"matching_rule\":{\"rule\":\"ACTION_ANY\",\"affected_account_id\":\"*.near\",\"status\":\"SUCCESS\"}}"
        //     },
        // });
        
        // // Let indexer initialize
        // await new Promise(resolve => setTimeout(resolve, 1000));

        // for (let i = 0; i < 3; i++) {
        //     try {
        //         await registry_contract.account.functionCall({
        //             contractId: registry_contract.contractId,
        //             methodName: 'add_user',
        //             args: {
        //                 "account_id": `new-user-${i}.test.near`,
        //             },
        //         });
        //     } catch (error) {
        //         console.log(error);
        //     }
        // }
        const query = `query MyQuery {
            dev_queryapi_test_near_test_indexer_indexers {
                methodname
              }
          }`;
        const result = await fetchGraphQL(query, {});
        const expectedMethodNames = [
            'add_user',
            'add_user',
            'add_user',
          ];
        const actualMethodNames = result.data.dev_queryapi_test_near_test_indexer_indexers.map(item => item.methodname);
        assert.deepStrictEqual(actualMethodNames, expectedMethodNames, 'The order or values of method names do not match');
    }, 30000);
});

async function fetchGraphQL(query, variables) {
    const response = await fetch('http://127.0.0.1:56413/v1/graphql', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'x-hasura-admin-secret': 'myadminsecretkey' // or use an appropriate auth token
      },
      body: JSON.stringify({
        query,
        variables
      })
    });
  
    return response.json();
  }