# NFT-demo for Blockchain Z-days

Link to conference agenda:

<https://community-z.com/events/blockchain-z-days-2021>

## Scripts

The `Makefile` contains several scripts to build, test and deploy the NFT smart contract.
Set the `NEAR_ACCOUNT` variable in `Makefile` to your test account to be able to mint and view data in the smart contract.
Here is the list of `make` scripts:

* `build`. Builds the NFT Demo using WASM target in release mode.
* `test`. Runs the unit tests for the NFT Demo.
* `deploy`. Deploys the smart contract using a development account.
* `metadata`. Retrieves the NFT metadata using the `nft_metadata` method.
* `mint`. Mints a test token using the `nft_mint` method.
* `view`. Fetches the newly minted token using the `nft_token` method.
* `clean`. Removes the `neardev` folder in case you want to redeploy the smart contract.
