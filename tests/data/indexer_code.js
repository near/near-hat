  function base64decode(encodedValue) {
    let buff = Buffer.from(encodedValue, "base64");
    return JSON.parse(buff.toString("utf-8"));
  }
  function convertNanosecondsToTimestamp(nanosecondsStr) {
    const nanoseconds = BigInt(nanosecondsStr);
    const milliseconds = nanoseconds / 1000000n;
    const date = new Date(Number(milliseconds));
    return date;
  }

  const QUERY_API_CONTRACT = "dev-queryapi.test.near";
  block
    .actions()
    .filter((action) => action.receiverId === QUERY_API_CONTRACT)
    .flatMap((action) =>
      action.operations
        .filter((operation) => operation["FunctionCall"] !== undefined)
        .map((operation) => operation["FunctionCall"])
        .map((functionCallOperation) => ({
          ...functionCallOperation,
          args: base64decode(functionCallOperation.args),
          methodName: functionCallOperation.methodName,
          receiptId: action.receiptId,
        }))
        .map((functionCall) => {
          console.log("Func: ", functionCall)
          console.log(functionCall.methodName)
        }));
  const queryApiContractTxs = block
    .actions()
    .filter((action) => action.receiverId === QUERY_API_CONTRACT)
    .flatMap((action) =>
      action.operations
        .filter((operation) => operation["FunctionCall"] !== undefined)
        .map((operation) => operation["FunctionCall"])
        .map((functionCallOperation) => ({
          ...functionCallOperation,
          args: base64decode(functionCallOperation.args),
          methodName: functionCallOperation.methodName,
          receiptId: action.receiptId,
        }))
        .filter((functionCall) => functionCall.methodName !== undefined)
        .map((functionCall) => {
          const functionName = functionCall.args.function_name || "no";
          const accountId = functionCall.args.account_id || "no";
          const methodName = functionCall.methodName;
          const signerId = action.signerId;
          return { accountId, functionName, methodName, signerId };
        })
    );
  if (queryApiContractTxs.length > 0) {
    console.log("Found QueryAPI Contract Activity...");
    const block_height = block.header().height;
    await Promise.all(
      queryApiContractTxs.map(
        async ({ accountId, functionName, methodName, signerId }) => {
          console.log(`Handling ${methodName} call from ${signerId} regarding ${functionName} function and ${accountId} account}`);
          try {
            await context.db.Indexers.insert({functionName, accountId, signerId, methodName, block_height});
          } catch (err) {
            console.log(
              `Error processing receipt at blockHeight: ${block_height}: ${err}`
            );
            return err;
          }
        }
      )
    );
  }