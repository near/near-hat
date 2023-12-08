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
        const base_account = await nearConnection.account("dev-queryapi.test.near");
        const registry_contract = new Contract(base_account, 'dev-queryapi.test.near', {
            viewMethods: ['read_indexer_function'],
            changeMethods: ['register_indexer_function'],
        });
        const code = readFileSync('data/indexer_code.js').toString();
        const schema = readFileSync('data/indexer_schema.sql').toString();
        const response = await registry_contract.account.functionCall({
            contractId: registry_contract.contractId,
            methodName: 'register_indexer_function',
            args: {
                "function_name": "test_indexer",
                "code": code,
                "schema": schema,
                "filter_json": "{\"indexer_rule_kind\":\"Action\",\"matching_rule\":{\"rule\":\"ACTION_ANY\",\"affected_account_id\":\"*.near\",\"status\":\"SUCCESS\"}}"
            },
        });
        // TODO: Get block height from response to filter result of later graphql call (To allow successive calls)
        
        // Let indexer initialize
        await new Promise(resolve => setTimeout(resolve, 1000));

        for (let i = 0; i < 3; i++) {
            await registry_contract.account.functionCall({
                contractId: registry_contract.contractId,
                methodName: 'add_user',
                args: {
                    "account_id": `new-user-${i}.test.near`,
                },
            });
        }
        const query = `query MyQuery {
            dev_queryapi_test_near_test_indexer_indexers {
              methodname
            }
          }`;
        const result = await fetchGraphQL(query, {});
        console.log(result);
        const expectedMethodNames = [
            'add_user',
            'add_user',
            'add_user',
          ];
        const actualMethodNames = result.data.dev_queryapi_test_near_test_indexer_indexers.map(item => item.methodname);
        assert(doLastElementsMatch(actualMethodNames, expectedMethodNames), 'The order of values of method names do not match');
    }, 30000);
});

function doLastElementsMatch(resultArray, expectedArray) {
    // Get the last elements of the result array based on the length of the expected array
    const lastElements = resultArray.slice(-expectedArray.length);

    // Compare the last elements with the expected array
    return lastElements.length === expectedArray.length &&
           lastElements.every((element, index) => element === expectedArray[index]);
}

async function fetchGraphQL(query, variables) {
    const response = await fetch('http://playground.nearhat/v1/graphql', {
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