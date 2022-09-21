# CosmWasm Osmosis

### References

- [CosmWasm Documentation](https://docs.cosmwasm.com/docs/1.0/)
- [Osmosis Deployment Guide](https://docs.osmosis.zone/cosmwasm/testnet/cosmwasm-deployment)
- [CosmWasm Optimizer](https://github.com/CosmWasm/rust-optimizer)
- https://github.com/osmosis-labs/osmosis-rust/tree/main/packages/osmosis-std
- https://github.com/osmosis-labs/osmosis-rust/tree/main/examples/cosmwasm
- https://github.com/CosmWasm/cw-plus
- https://github.com/CosmWasm/cw-tokens/tree/main/contracts/cw20-staking

## Deployment

- TODO: write a script to deploy

### Optimize

```shell
docker run --rm -v "$(pwd)":/code \
    --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
    --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
    cosmwasm/rust-optimizer:0.12.8
```

### Upload

```
osmosisd tx wasm store artifacts/vault.wasm --from wallet --gas-prices 0.1uosmo --gas auto --gas-adjustment 1.3 -y --output json -b block
```

Response

```json
{
	// ...
	"logs": [
		{
			// ...
			"events": [
				// ...
				{
					"type": "store_code",
					"attributes": [
						{
							"key": "code_id",
							"value": "1531" // this value
						}
					]
				}
			]
		}
	]
}
```

Get the `store_code` event's `code_id` in the JSON response for the step:

### Instantiate

```shell
CODE_ID='1531'
INIT='{"pool_id":2}'
osmosisd tx wasm instantiate $CODE_ID $INIT \
 --from wallet --label "vault" --gas-prices 0.025uosmo --gas auto --gas-adjustment 1.3 -b block -y --no-admin
```

### Read

```shell
QUERY='{"get_count": {}}'
osmosisd query wasm contract-state smart $CONTRACT_ADDR $QUERY --output json
```

### Write

Join

```shell
JOIN='{"join": {}}'
AMOUNT='10000000uosmo'
osmosisd tx wasm execute $CONTRACT_ADDR $JOIN --from wallet --gas-prices 0.025uosmo --gas auto --gas-adjustment 1.3 --amount $AMOUNT -y
```

Compound

```shell
COMPOUND='{"compound": { "min_shares": 1 }}'
osmosisd tx wasm execute $CONTRACT_ADDR $COMPOUND --from wallet --gas-prices 0.025uosmo --gas auto --gas-adjustment 1.3 -y
```
