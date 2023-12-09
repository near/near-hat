let transactions = [];
// EVENT_JSON:{"standard":"nep141","version":"1.0.0","event":"ft_mint","data":[{"owner_id":"alice.near","amount":"100000000"}]}
block.events()
  .filter((event) => (event && event.rawEvent && event.rawEvent.standard))
  .map((event) => {
    let res = JSON.parse(event.rawEvent.standard);
    console.log("event:", res);
    res.data.forEach(eventItem => {
      transactions.push({
        event: res.event, 
        amount: eventItem.amount, 
        from_account: eventItem.old_owner_id || (res.event === 'ft_burn' ? eventItem.owner_id : null), 
        to_account: eventItem.new_owner_id || (res.event === 'ft_mint' ? eventItem.owner_id : null)
      });
    })
  });
console.log(transactions);
transactions.forEach(async (tx) => {
  await context.db.Indexers.insert({amount: tx.amount, block_height: block.blockHeight, event: tx.event, from_account: tx.from_account, to_account: tx.to_account});
});