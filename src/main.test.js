jest.setTimeout(20000);

beforeAll(async function () {
  // NOTE: nearlib and nearConfig are made available by near-cli/test_environment
  const near = await nearlib.connect(nearConfig)
  window.accountId = nearConfig.contractName
  window.contract = await near.loadContract(nearConfig.contractName, {
    viewMethods: ['get_stats'],
    changeMethods: ['vote_for'],
    sender: window.accountId
  })
})

test('set_then_get_stats', async () => {
  let ratings = [1.0, 4.5, 2.0, 0.5, 1.5, 3.0, 5.0];
  for (let rating of ratings)
    await window.contract.vote_for({ account_id: "alice_near", rating })
  const myRating = await window.contract.get_stats({ account_id: window.accountId })
  const theirRating = await window.contract.get_stats({ account_id: "alice_near" })
  expect(myRating).toEqual({rating: 2.0, given: 7, received: 1})
  expect(theirRating).toMatchObject({
    rating: ratings.reduce((a, b, i) => ((a * (i + 1)) + (b + 2) / 2) / (i + 2), 2),
    given: 0,
    received: 8
  })
})
