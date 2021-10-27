jest.setTimeout(20000);

beforeAll(async function () {
  // NOTE: nearlib and nearConfig are made available by near-cli/test_environment
  const near = await nearlib.connect(nearConfig)
  window.accountId = nearConfig.contractName
  window.contract = await near.loadContract(nearConfig.contractName, {
    viewMethods: ['get_rating'],
    changeMethods: ['vote_for'],
    sender: window.accountId
  })
})

test('set_then_get_rating', async () => {
  let ratings = [1.0, 4.5, 2.0, 0.5, 1.5, 3.0];
  for (let rating of ratings)
    await window.contract.vote_for({ account_id: window.accountId, rating })
  const rating = await window.contract.get_rating({ account_id: window.accountId })
  expect(rating).toBeCloseTo(ratings.reduce((a, b) => a + b, 0) / ratings.length)
})
