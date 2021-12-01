# near-nosedive

NEAR smart contract implementing a rating system between NEAR accounts.
Inspired by Black Mirror's [NoseDive](https://en.wikipedia.org/wiki/Nosedive_(Black_Mirror)) episode.
The contract in [contract/src/lib.rs](contract/src/lib.rs) provides methods to register one's own account, vote a registered account, view account state, etc.

Front end isn't implemented yet, but given that all the [integration tests](src/main.test.js) checkout, UI is a non-issue.

## Contract API

Functions with `&mut self` are change methods, requiring the caller to sign a transactio. While those without it are view methods and can be called without needing a transaction.

### <a id='fn:register'></a> register(&mut self)

> * Panics if the account already exists.
> * This requires a signed transaction.
> * Free to call (this function call requires no transfer of funds).

Registers the signer of the transaction on the smart contract. Accounts cannot be rated or queried if they haven't first registered. Registered accounts are automatically given `2` points on the rating system.

### status(account_id)

* `account_id`: &lt;[AccountId][account-id]&gt;
* Returns: &lt;[object]&gt;
  * `rating`: &lt;[float][number]&gt; Floating-point rating of the account from `0` to `5`.
  * `given`: &lt;[number]&gt; How many ratings this account has given out.
  * `received`: &lt;[number]&gt; How many ratings this account has received from other accounts.

> * This requires the account to be [registered](#fn:register) on the contract.

Queries the rating status of a single account.

### rating_timestamps(a, b)

* `a`: &lt;[AccountId][account-id]&gt;
* `b`: &lt;[AccountId][account-id]&gt;
* Returns: &lt;[object]&gt;
  * `a_to_b`: &lt;[number]&gt; | &lt;[null]&gt; Nanosecond timestamp for when last the account `a` rated `b`, `null` if never.
  * `b_to_a`: &lt;[number]&gt; | &lt;[null]&gt; Nanosecond timestamp for when last the account `b` rated `a`, `null` if never.

> * This requires both accounts to be [registered](#fn:register) on the contract.

Queries most recent timestamps for both accounts rating each other.

### rate(&mut self, account_id, rating)

* `account_id`: &lt;[AccountId][account-id]&gt; The account you want to rate.
* `rating`: &lt;[float][number]&gt; Floating-point rating between `0` and `5`, but by multiples of `.5`.

> * You cannot rate yourself.
> * This requires both the rater and the account being rated to be [registered](#fn:register) on the contract.
> * Rating must be a multiple of `.5` between `0` and `5`. (`2.5`, `4.0`, etc..)
> * By default, you can only rate an account once every 5 minutes.
> * This requires a signed transaction.
> * Free to call (this function call requires no transfer of funds).

Allows one account to rate another, on a scale of `0` to `5`.

### patch_state(&mut self, patches)

* `patches`: &lt;[object]&gt;
  * `voting_interval`: &lt;[object]&gt; | &lt;[null]&gt; (use `null` to remove the interval)
    * `secs`: &lt;[number]&gt; Duration in seconds before an account is allowed to vote the same other account again. Default: 5 minutes
    * `msg`: &lt;[string]&gt; Message to report when an account tries to rate the same other account before the expiration of that waiting interval. Default: `"you can't vote the same account more than once in 5 minutes"`

> * Only the account the contract is deployed on may call this function.
> * This requires a signed transaction.
> * Free to call (this function call requires no transfer of funds).

Aids the contract deployer in dynamically configuring the deployed contract.
A "settings" helper if you will.

<details>

<summary> This app was initialized with <a href="https://github.com/near/create-near-app">create-near-app</a> </summary>

## Quick Start

To run this project locally:

1. Prerequisites: Make sure you've installed [Node.js] ≥ 12
2. Install dependencies: `yarn install`
3. Run the local development server: `yarn dev` (see `package.json` for a
   full list of `scripts` you can run with `yarn`)

Now you'll have a local development environment backed by the NEAR TestNet!

Go ahead and play with the app and the code. As you make code changes, the app will automatically reload.

## Exploring The Code

1. The "backend" code lives in the `/contract` folder. See the README there for
   more info.
2. The frontend code lives in the `/src` folder. `/src/index.html` is a great
   place to start exploring. Note that it loads in `/src/index.js`, where you
   can learn how the frontend connects to the NEAR blockchain.
3. Tests: there are different kinds of tests for the frontend and the smart
   contract. See `contract/README` for info about how it's tested. The frontend
   code gets tested with [jest]. You can run both of these at once with `yarn
   run test`.

## Deploy

Every smart contract in NEAR has its [own associated account][NEAR accounts]. When you run `yarn dev`, your smart contract gets deployed to the live NEAR TestNet with a throwaway account. When you're ready to make it permanent, here's how.

### Step 0: Install near-cli (optional)

[near-cli] is a command line interface (CLI) for interacting with the NEAR blockchain. It was installed to the local `node_modules` folder when you ran `yarn install`, but for best ergonomics you may want to install it globally:

    yarn install --global near-cli

Or, if you'd rather use the locally-installed version, you can prefix all `near` commands with `npx`

Ensure that it's installed with `near --version` (or `npx near --version`)

### Step 1: Create an account for the contract

Each account on NEAR can have at most one contract deployed to it. If you've already created an account such as `your-name.testnet`, you can deploy your contract to `nosedive-rs.your-name.testnet`. Assuming you've already created an account on [NEAR Wallet], here's how to create `nosedive-rs.your-name.testnet`:

1. Authorize NEAR CLI, following the commands it gives you:

      near login

2. Create a subaccount (replace `YOUR-NAME` below with your actual account name):

      near create-account nosedive-rs.YOUR-NAME.testnet --masterAccount YOUR-NAME.testnet

### Step 2: set contract name in code

Modify the line in `src/config.js` that sets the account name of the contract. Set it to the account id you used above.

    const CONTRACT_NAME = process.env.CONTRACT_NAME || 'nosedive.YOUR-NAME.testnet'

### Step 3: deploy

One command:

    yarn deploy

As you can see in `package.json`, this does two things:

1. builds & deploys smart contract to NEAR TestNet
2. builds & deploys frontend code to GitHub using [gh-pages]. This will only work if the project already has a repository set up on GitHub. Feel free to modify the `deploy` script in `package.json` to deploy elsewhere.

## Troubleshooting

On Windows, if you're seeing an error containing `EPERM` it may be related to spaces in your path. Please see [this issue](https://github.com/zkat/npx/issues/209) for more details.

</details>

## License

[Apache 2.0][license] © **Miraculous Owonubi** ([@miraclx][author-url]) \<omiraculous@gmail.com\>

  [account-id]: https://docs.near.org/docs/concepts/account#account-id-rules
  [object]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object
  [number]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number
  [string]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String
  [null]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/null

  [license]: LICENSE "Apache 2.0 License"
  [author-url]: https://github.com/miraclx

  [Node.js]: https://nodejs.org/en/download/package-manager/
  [jest]: https://jestjs.io/
  [NEAR accounts]: https://docs.near.org/docs/concepts/account
  [NEAR Wallet]: https://wallet.testnet.near.org/
  [near-cli]: https://github.com/near/near-cli
  [gh-pages]: https://github.com/tschaub/gh-pages
