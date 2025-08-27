# DAO with NFT Verification

**Important Note**: This code was used for a conference presentation, and hence has some sharp edges. It is not secure code.
For example, the NFT verification has a vulnerability where verifying the NFT on Asset Hub (AH) is insecure. This is because
ISMP requires finalization on the Polkadot relay chain, which means AH will be a few blocks ahead and the NFT may not even belong
to the user anymore.

# Guide for the Multichain DAO Smart Contract

This guide provides an overview of an Ink! smart contract named `Dao`, designed to showcase cross-chain interactions the messaging API with ISMP. The contract enables users to register NFTs (Non-Fungible Tokens) from another parachain, verify their ownership, and mint corresponding NFTs on the local chain.
---

<aside>
ℹ️

This a proof-of-concept presented at Sub0

- Contract: [https://github.com/r0gue-io/pop-node/blob/daan/sub0/pop-api/integration-tests/contracts/dao/lib.rs](https://github.com/r0gue-io/pop-node/blob/daan/sub0/pop-api/integration-tests/contracts/dao/lib.rs)
- Demo: [https://www.youtube.com/watch?v=Ll3nlshPg_U](https://www.youtube.com/watch?v=Ll3nlshPg_U)
</aside>

## 1. Introduction

The `Dao` smart contract uses:

- **NFT Registration and Verification**: Users can register NFTs from a specified parachain, verify their ownership, and mint equivalent NFTs on the local chain.
- **Register Callbacks:** The messaging API allows registering callbacks for automatically returning the response of the message.
- **Event-Driven Feedback**: The contract emits events to notify users of key actions and their outcomes.

This contract leverages the `pop_api` crate for runtime interactions.

---

## 2. Setup

Install the pop cli with relaying:
```
cargo install --git https://github.com/r0gue-io/pop-cli --branch interpop
```

Go to the master branch of Pop Network, build the node with ismp and launch a test network:
```
cargo build --release --features=ismp
pop up network -f networks/testnet.toml
```
### Deployment

Deploy the contract:
```
pop up -c new -u ws://127.0.0.1:9944 -v 100000000000000
```
- **Provide Sufficient Endowment**: Ensure the contract is deployed with enough funds, as the constructor transfers a portion of the endowment to the contract’s account on the destination parachain.
- **Call the Constructor**: The `new()` constructor initializes the contract and performs the following actions:
    - Initializes an `NftVerifier` instance with a hardcoded parachain ID (`1000`) and collection ID (`0`) for verifying NFTs.
    - Creates a new NFT collection on the local chain using the `nonfungibles` API.
 
### Configuration

- **Parachain and Collection IDs**: The `NftVerifier` uses a hardcoded parachain ID of `1000` and collection ID of `0`. If your use case requires different values, modify the `NftVerifier::new(1000, 0)` call in the constructor.

To create collection `0` and mint a item `1` on Asset Hub use:
```
pop call chain --pallet Nfts --function create --args "Id(15oF4uVJwmo4TdGW7VfQxNLavjCXviqxT9S1MgbjMNHr6Sp5)" "{settings: 0, max_supply: None(), mint_settings: {mint_type: Issuer(), price: None(), start_block: None(), end_block: None(), default_item_settings: 0}}" --url ws://127.0.0.1:9977/ --suri //Alice

pop call chain --pallet Nfts --function create --args "Id(15oF4uVJwmo4TdGW7VfQxNLavjCXviqxT9S1MgbjMNHr6Sp5)" "{settings: 0, max_supply: None(), mint_settings: {mint_type: Issuer(), price: None(), start_block: None(), end_block: None(), default_item_settings: 0}}" --url ws://127.0.0.1:9977/ --suri //Alice
```

---

## 3. Registering an NFT

### Function: `register(height: u32, item: ItemId)`

To register with NFT item ID `1` at block height `1000`:

```
pop call contract --contract 5GLUXEDyR7DqsA2FFpFsBcGBa2G4n8jJwPqebt9LTEDhgfM9 --message register --args "69", "1" --url ws://localhost:9944/ --suri //Alice --execute
```
- **Purpose**: Initiates the verification of an NFT’s ownership on the target parachain (ID `1000`).
- **Parameters**:
    - `height: u32`: The block height at which to query the NFT ownership on the other parachain.
    - `item: ItemId`: The ID of the NFT to register (a `u32` value).
- **Process**:
    - The caller’s `AccountId` is retrieved via `self.env().caller()`.
    - The `NftVerifier::verify()` function generates a storage key and sends an ISMP `get` request to query the NFT ownership on the target parachain.
    - The registration status for the `item` is set to `Pending` in the `registered_items` mapping.
- **Events**: Emits a `RegistrationRequested` event with the caller’s `AccountId` and the `ItemId`.

The event should look like:
```
Event Ismp ➜ Request
│           dest_chain: Polkadot(1000)
│           source_chain: Polkadot(4001)
│           request_nonce: 0
│           commitment: 0x2bdb91ce388cbcbb9beb74e7be22599da1c9c46c468b45b600b245bb3a11a446
---
```

## 4. Completing Registration

Now use the Pop CLI to relay the message with the commitment from above event:
```
pop relay get --source ws://127.0.0.1:9944 --dest ws://127.0.0.1:9977 --commitment 0x2bdb91ce388cbcbb9beb74e7be22599da1c9c46c468b45b600b245bb3a11a446
```

This should then call into `complete_registration`

### Callback: `complete_registration(id: MessageId, values: Vec<StorageValue>)`

- **Selector**: `0x57ad942b`
- **Purpose**: Processes the asynchronous response from the NFT verification request.
- **Parameters**:
    - `id: MessageId`: The ID of the verification request (a `u64` value).
    - `values: Vec<StorageValue>`: The storage values returned from the ISMP `get` request.
- **Process**:
    - Retrieves the `(AccountId, ItemId)` pair associated with the `MessageId` from the `requests` mapping.
    - Checks if the NFT is owned by the user (i.e., `values[0].value.is_some()`):
        - If verified, mints a new NFT in the local collection with the next available `ItemId` (`next_item_id`), increments `next_item_id`, and updates the `registered_items` status to `Used`.
        - If not verified, no new NFT is minted.
    - Emits a `RegistrationCompleted` event with the result.
- **Events**:
    - `RegistrationCompleted { account: AccountId, verified_item: ItemId, membership: Option<ItemId> }`, where `membership` is `Some(ItemId)` if an NFT was minted, or `None` if verification failed.

### Outcome

- Success: A new NFT is minted, and you receive a membership token.
- Failure: No NFT is minted (e.g., if the NFT isn’t owned by the caller at the specified height).
