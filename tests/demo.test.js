import { connect, Contract, keyStores, utils } from 'near-api-js';
import { restoreTestAccountKeys, getOrCreateAccount, registerIndexer, fetchGraphQL } from './testUtils.js'
import assert from 'assert';

describe('Hackathon Demo', () => {
    const connectionConfig = {
        networkId: "localnet",
        nodeUrl: "http://rpc.nearhat",
        walletUrl: "NONE",
        keyStore: new keyStores.InMemoryKeyStore()
    };

    test('Transaction events queryable after successful contract executions', async () => {
        const near = await connect(connectionConfig);
        restoreTestAccountKeys(near);
        
        await registerIndexer("usdt_transactions", 'data/indexer_code.js', 'data/indexer_schema.sql', "*.near", near);

        const multisafe = await getOrCreateAccount(near, "multisafe.near", 20);
        const usdtOwner = await getOrCreateAccount(near, "tether.multisafe.near", 5);
        const alice = await getOrCreateAccount(near, "alice.near", 2);
        const bob = await getOrCreateAccount(near, "bob.near", 2);
        
        const adminUsdtContractSigner = new Contract(usdtOwner, 'usdt.tether-token.near', {
          viewMethods: ['ft_balance_of'],
          changeMethods: ['mint', 'storage_deposit'],
        });

        // Cover storage deposit
        await adminUsdtContractSigner.storage_deposit({
          args: { "account_id": alice.accountId },
          gas: "300000000000000",
          amount: "10000000000000000000000"
        });
        await adminUsdtContractSigner.storage_deposit({
          args: { "account_id": bob.accountId },
          gas: "300000000000000",
          amount: "10000000000000000000000"
        });

        // Mint 100 USDT to Alice
        const response = await adminUsdtContractSigner.mint({
            account_id: alice.accountId,
            amount: "100000000"
        });
        assert.strictEqual(
          await adminUsdtContractSigner.ft_balance_of(
            { account_id: alice.accountId }
          ),"100000000", "Alice should have 100 USDT to start");

        // Transfer 50 USDT from Alice to Bob
        const aliceUsdtContractSigner = new Contract(alice, 'usdt.tether-token.near', {
          viewMethods: [],
          changeMethods: ['ft_transfer', 'storage_deposit'],
        });
        await aliceUsdtContractSigner.ft_transfer(
          {
            receiver_id: bob.accountId,
            amount: "50000000"
          }, 
          "300000000000000",
          "1"
        );

        // Verify Alice and Bob both have 50 USDT
        assert.strictEqual(await adminUsdtContractSigner.ft_balance_of(
          { account_id: alice.accountId }
        ), "50000000", "Alice should have 50 USDT after transfer");
        assert.strictEqual(await adminUsdtContractSigner.ft_balance_of(
          { account_id: bob.accountId }
        ), "50000000", "Bob should have 50 USDT after transfer");

        // // Wait for indexer to index latest transactions
        console.log("Waiting for indexer to index latest transactions...");
        await new Promise(resolve => setTimeout(resolve, 3000));

        // assert QueryAPI has mint, transfer
        const query = `query MyQuery {
          dev_queryapi_test_near_usdt_transactions_usdt_transactions {
            event
          }
        }`;
        const result = await fetchGraphQL(query, {});
        const events = result.data.dev_queryapi_test_near_usdt_transactions_usdt_transactions;
        const hasFtMint = events.some(e => e.event === 'ft_mint');
        assert(hasFtMint, "'ft_mint' event not found");

        // Check if 'ft_transfer' is present
        const hasFtTransfer = events.some(e => e.event === 'ft_transfer');
        assert(hasFtTransfer, "'ft_transfer' event not found");
    }, 30000);
});

function doLastElementsMatch(resultArray, expectedArray) {
    // Get the last elements of the result array based on the length of the expected array
    const lastElements = resultArray.slice(-expectedArray.length);

    // Compare the last elements with the expected array
    return lastElements.length === expectedArray.length &&
           lastElements.every((element, index) => element === expectedArray[index]);
}