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
    await window.contract.vote_for({ account_id: "alice_near", rating })
  const myRating = await window.contract.get_rating({ account_id: window.accountId })
  const theirRating = await window.contract.get_rating({ account_id: "alice_near" })
  expect(myRating).toEqual(2.0)
  expect(theirRating).toBeCloseTo(ratings.reduce((a, b, i) => ((a * (i + 1)) + (b * 2) / 5) / (i + 2), 2))
})
