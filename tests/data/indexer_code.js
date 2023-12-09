let transactions = [];
block.events()
  .filter((event) => (event && event.rawEvent && event.rawEvent.standard))
  .map((event) => {
    let standardEvent = JSON.parse(event.rawEvent.standard);
    console.log("event:", standardEvent);
    standardEvent.data.forEach(eventItem => {
      transactions.push({
        event: standardEvent.event, 
        amount: eventItem.amount, 
        from_account: eventItem.old_owner_id || (standardEvent.event === 'ft_burn' ? eventItem.owner_id : null), 
        to_account: eventItem.new_owner_id || (standardEvent.event === 'ft_mint' ? eventItem.owner_id : null)
      });
    })
  });
console.log(transactions);
transactions.forEach(async (tx) => {
  await context.db.UsdtTransactions.insert({amount: tx.amount, block_height: block.blockHeight, event: tx.event, from_account: tx.from_account, to_account: tx.to_account});
});