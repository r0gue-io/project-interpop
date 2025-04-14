# Swap on Hydration Example

## Description

### Reserve Transfer Instructions
```
WithdrawAsset
InitiateReserveWithdraw(Parachain(intermediary_hop))
DepositReserveAsset
```

### Multi-hop Swapping Instructions

- `from_para`: The parachain ID from which the transfer will originate. (e.g. Pop Network - 4001)
- `to_para`: The parachain ID to which the transfer will be routed.
- `intermediary_hop`: The parachain ID of the intermediary parachain. (e.g. Asset Hub - 1000)
- `swap_chain`: The parachain ID of the swap chain. (e.g. Hydration - 2043)
- `amount`: The amount of tokens to be transferred.
- `destination_account`: The destination account on the `to_para` parachain.

To transfer from `from_para` to `intermediary_hop`:
```js
WithdrawAsset
InitiateReserveWithdraw(Parachain(intermediary_hop))
DepositReserveAsset(Parachain(swap_chain))
```
Swap on Hydration then reserve transfer to the `intermediary_hop` then the `to_para`:
```js
BuyExecution
ExchangeAsset
InitiateReserveTransfer(Parachain(to_para))
DepositReserveAsset(AccountId32(destination_account))
```
Full flow breakdown:
```json
POP (4001)
  └──[reserve transfer + DepositReserveAsset]──▶ Asset Hub (1000)
         └──[DepositReserveAsset]──▶ Hydration (2043)
              └──[BuyExecution + ExchangeAsset]
                  └──[InitiateReserveWithdraw]──▶ Asset Hub (1000)
                      └──[DepositReserveAsset]──▶ POP (4001)
                          └── deposit to final beneficiary
```


## Contract development

To build a contract

```
pop b -r
```

To deploy a contract on Pop Network Testnet (Local)

```
pop deploy --url=ws://127.0.0.1:9944 --suri //Alice --salt 1234
```

To deploy a contract on Pop Network Testnet (Paseo)

```
pop deploy --url=wss://rpc3.paseo.popnetwork.xyz --suri //Alice --salt 1234
```
