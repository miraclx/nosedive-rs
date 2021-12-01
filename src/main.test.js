const {log} = require("console");
const {promisify} = require("util");

jest.setTimeout(100_000);

const sleep = promisify(setTimeout);

let NEAR, sys, keyStore, masterKeyPair, masterAccount;

beforeAll(async function () {
  NEAR = await nearlib.connect(nearConfig);
  keyStore = nearConfig.deps.keyStore;
  masterKeyPair = await keyStore.getKey(nearConfig.networkId, nearConfig.contractName);
  masterAccount = await NEAR.account(nearConfig.contractName);
  sys = await stage(await account(nearConfig.contractName, true));
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
        10n ** 25n
      );
      await keyStore.setKey(nearConfig.networkId, account.accountId, masterKeyPair);
    }
  }
  throw new Error(`Could not find nor create the account: ${accountId}`);
}

async function stage(account, flags = "rw") {
  return new nearlib.Contract(account, nearConfig.contractName, {
    ...(
      flags.includes('r') ? { viewMethods: ['status', 'rating_timestamps'] } : {}
    ),
    ...(
      flags.includes('w') ? { changeMethods: ['register', 'rate', 'patch_state'] } : {}
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

test('no account', async () => {
  let carol = await account("carol");
  // await carol.register();
  await expect(async () => {
    await sys.status({account_id: carol.accountId})
  }).rejects.toThrow("account does not exist on this service");

  let carolsMom = await stage(await account("carols-mom"));
  await carolsMom.register();
  await expect(async () => {
    await carolsMom.rate({account_id: carol.accountId, rating: 5.0})
  }).rejects.toThrow("account does not exist on this service");
});

test('rate then view status', async () => {
  await sys.patch_state({patches: {voting_interval: null}});

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

test('lookup timestamps', async () => {
  let fiona = await stage(await account("fiona"));
  await fiona.register();

  let gary = await stage(await account("gary"));
  await gary.register();

  expect(await sys.rating_timestamps({
    a: fiona.account.accountId,
    b: gary.account.accountId
  })).toEqual({ a_to_b: null, b_to_a: null });

  let start = Date.now();

  await fiona.rate({account_id: gary.account.accountId, rating: 3.0});
  let fiona_gary = Date.now();

  await sleep(2000);

  await gary.rate({account_id: fiona.account.accountId, rating: 3.0});
  let gary_fiona = Date.now();

  let ratings = await sys.rating_timestamps({
    a: fiona.account.accountId,
    b: gary.account.accountId
  });

  expect(ratings.a_to_b / 1_000_000).toBeGreaterThanOrEqual(start);
  expect(ratings.a_to_b / 1_000_000).toBeLessThanOrEqual(fiona_gary);
  expect(ratings.b_to_a / 1_000_000).toBeGreaterThanOrEqual(fiona_gary);
  expect(ratings.b_to_a / 1_000_000).toBeLessThanOrEqual(gary_fiona);
});

test('patch auth', async () => {
  await expect(async () => {
    let helen = await stage(await account("helen"));
    await helen.patch_state({patches: {voting_interval: null}})
  }).rejects.toThrow("only the account that deployed this contract is permitted to call this method");

  await sys.patch_state({patches: {voting_interval: null}})
});

test('interval', async () => {
  let ivan = await stage(await account("ivan"));
  await ivan.register();

  let janet = await stage(await account("janet"));
  await janet.register();

  await sys.patch_state({
    patches: {
      voting_interval: {
        secs: 60,
        msg: "all you had to do was wait a minute"
      }
    }
  });

  await ivan.rate({account_id: janet.account.accountId, rating: 4.5});
  await expect(async () => {
    await ivan.rate({account_id: janet.account.accountId, rating: 4.5});
  }).rejects.toThrow("all you had to do was wait a minute");

  await sys.patch_state({
    patches: {
      voting_interval: {
        secs: 2,
        msg: "wait at least two seconds to be allowed to vote the same person again"
      }
    }
  });

  await ivan.rate({account_id: janet.account.accountId, rating: 4.5});
  await sleep(3);
  await ivan.rate({account_id: janet.account.accountId, rating: 4.5});
});
