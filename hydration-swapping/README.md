# Swap on Hydration Example

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

## Frontend development

```
pnpm papi add -w ws://127.0.0.1:9944 popTestnetLocal
pnpm papi ink add ../target/ink/hydration_swapping.json
```
