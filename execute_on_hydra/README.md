# Simple Example For Executing and Reading from Hydra

## Setup
- Install Pop CLI, use the `interpop` branch. This includes a manual ISMP relayer (self-relaying)
```
cargo install --git https://github.com/r0gue-io/pop-cli --branch interpop
```

## Steps

This is a guide to execute a smart contract that generically uses Hydration. It supports an arbitrary encoded call data to perform a `Transact` on Hydration.
Additionally, it allows querying an arbitrary storage key from the contract.

The smart contract on Pop will control an account on Hydration which will require HDX to pay the fees. 

1. Add existing contract on Pop https://contracts.onpop.io/add-contract  
  - Use this metadata [./execute_on_hydra_metadata.json](./execute_on_hydra_metadata.json)
  - Use this contract address: `13ekCGKXooHstd3C4kaJMyX5KAsqW6P4W8GJzUCEfApNaxD9`

![add contract](./images/add-contract.png "add existing contract")

2. On [Hydration](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Fpaseo-rpc.play.hydration.cloud#/extrinsics) create an extrinsic and copy its encoded call data

![copy encoded call data](./images/copy-encoded-call.png "copy encoded call data")

3. Make sure the contract's account on Hydration has HDX to pay fees. You can find this account in the "Cross Chain Address" tab.

![contract address](./images/contract-address.png "contract address")

4. Create the smart contract call at https://contracts.onpop.io/contract/13ekCGKXooHstd3C4kaJMyX5KAsqW6P4W8GJzUCEfApNaxD9
  - Use the encoded call data from step 2

![smart contract executes](./images/sc-executes-encoded-call.png "smart contract executes encoded call data")
  - Verify any events on Hydration to make sure the transaction was successful. In further examples we can use query responses from XCM and callback the contract.

5. Get storage key to query from Hydration https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Fpaseo-rpc.play.hydration.cloud#/chainstate

![storage key](./images/storage-key.png "storage key")

6. Get the latest ISMP height (block number) for Hydration. You can do check this from Pop's events. 

![updated event](./images/ismp-height.png "ISMP height")

7. Create the smart contract call at https://contracts.onpop.io/contract/13ekCGKXooHstd3C4kaJMyX5KAsqW6P4W8GJzUCEfApNaxD9
  - Use the storage key from step 5
  - Use the ISMP height from step 6

![query storage](./images/query-storage.png "query storage")

8. Get ISMP commitment from event

   ![get ISMP commitment](./images/commitment.png "get ISMP commitment")

9. Self relay the ISMP data to Pop (make sure you are using pop cli on `interpop` branch).
```
# Only Pop's rpc1 and rpc3 nodes support ISMP, right now.
pop relay get --source wss://rpc1.paseo.popnetwork.xyz --dest wss://paseo-rpc.play.hydration.cloud --commitment <COMMITMENT FROM STEP 8>
```
![pop cli relay](./images/pop-cli-relay.png "pop cli relay output")
