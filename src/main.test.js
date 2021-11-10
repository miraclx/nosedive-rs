const {log} = require("console");

jest.setTimeout(30000);

let NEAR, sys, keyStore, masterKeyPair, masterAccount;

beforeAll(async function () {
  NEAR = await nearlib.connect(nearConfig);
  keyStore = nearConfig.deps.keyStore;
  masterKeyPair = await keyStore.getKey(nearConfig.networkId, nearConfig.contractName);
  masterAccount = await NEAR.account(nearConfig.contractName);
  sys = await stage(await account(nearConfig.contractName, true), 'r');
});

async function account(accountId, qualified = false) {
  let account = await NEAR.account(qualified ? accountId : [accountId, masterAccount.accountId].join('.'));
  for (i=5;i>0;i-=1) {
    try {
      await account.state();
      return account;
    } catch {
      log(`[${nearConfig.networkId}] Creating an account on the network for [${account.accountId}]`);
      await masterAccount.createAccount(
        account.accountId,
        masterKeyPair.getPublicKey(),
        10n ** 26n
      );
      await keyStore.setKey(nearConfig.networkId, account.accountId, masterKeyPair);
    }
  }
  throw new Error(`Could not find nor create the account: ${accountId}`);
}

async function stage(account, flags = "rw") {
  return new nearlib.Contract(account, nearConfig.contractName, {
    ...(
      flags.includes('r') ? { viewMethods: ['get_stats'] } : {}
    ),
    ...(
      flags.includes('w') ? { changeMethods: ['register', 'vote_for'] } : {}
    )
  });
}

test('default', async () => {
  let alice = await stage(await account("alice"));
  await alice.register();
  let aliceStats = await sys.get_stats({account_id: alice.account.accountId});
  expect(aliceStats).toEqual({rating: 2.0, given: 0, received: 1});
});

test('single entry', async () => {
  let bob = await stage(await account("bob"));
  await bob.register();
  await expect(async () => {
    await bob.register()
  }).rejects.toThrow("this account has already been registered");
});

test('set_then_get_stats', async () => {
  let carol = await stage(await account("carol"));
  await carol.register();
  // --
  let derek = await stage(await account("derek"));
  await derek.register();

  let ratings = [1.0, 4.5, 2.0, 0.5, 1.5, 3.0, 5.0];
  for (let rating of ratings)
    await carol.vote_for({account_id: derek.account.accountId, rating});
  let carolStats = await sys.get_stats({account_id: carol.account.accountId});
  let derekStats = await sys.get_stats({account_id: derek.account.accountId});
  expect(carolStats).toEqual({rating: 2.0, given: 7, received: 1})
  expect(derekStats).toMatchObject({
    rating: ratings.reduce((a, b, i) => ((a * (i + 1)) + (b + 2) / 2) / (i + 2), 2),
    given: 0,
    received: 8
  })
});
