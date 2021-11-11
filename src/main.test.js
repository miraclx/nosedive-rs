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
      flags.includes('r') ? { viewMethods: ['status'] } : {}
    ),
    ...(
      flags.includes('w') ? { changeMethods: ['register', 'rate'] } : {}
    )
  });
}

test('default', async () => {
  let alice = await stage(await account("alice"));
  await alice.register();
  let aliceStats = await sys.status({account_id: alice.account.accountId});
  expect(aliceStats).toEqual({rating: 2.0, given: 0, received: 1});
});

test('single entry', async () => {
  let bob = await stage(await account("bob"));
  await bob.register();
  await expect(async () => {
    await bob.register()
  }).rejects.toThrow("this account has already been registered");
});

test('no-account', async () => {
  let carol = await account("carol");
  await expect(async () => {
    await sys.status({account_id: carol.accountId})
  }).rejects.toThrow("account does not exist on this service");

  let carolsMom = await stage(await account("carols-mom"));
  await carolsMom.register();
  await expect(async () => {
    await carolsMom.rate({account_id: carol.accountId, rating: 5.0})
  }).rejects.toThrow("account does not exist on this service");
});

test('rate_then_view_status', async () => {
  let derek = await stage(await account("derek"));
  await derek.register();

  let emily = await stage(await account("emily"));
  await emily.register();

  let ratings = [1.0, 4.5, 2.0, 0.5, 1.5, 3.0, 5.0];
  for (let rating of ratings)
    await derek.rate({account_id: emily.account.accountId, rating});
  let derekStats = await sys.status({account_id: derek.account.accountId});
  let emilyStats = await sys.status({account_id: emily.account.accountId});
  expect(derekStats).toEqual({rating: 2.0, given: 7, received: 1})
  expect(emilyStats).toMatchObject({
    rating: ratings.reduce((a, b, i) => ((a * (i + 1)) + (b + 2) / 2) / (i + 2), 2),
    given: 0,
    received: 8
  })
});
