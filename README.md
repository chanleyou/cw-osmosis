# CosmWasm Osmosis

## Todos

- [ ] Lockup
- [ ] Handle pools with multiple rewards
- [ ] CW20 staking asset / share model
- [ ] Whitelist
- [ ] Queries (dependent on Osmosis v12 release)
- [ ] Tests
- [ ] Schema

## Testnet Deployment

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

Get the `store_code` event's `code_id` in the JSON response for the step:

```json
{
	"logs": [
		{
			"events": [
				{
					"type": "store_code",
					"attributes": [
						{
							"key": "code_id",
							"value": "1793" // this value
						}
					]
				}
			]
		}
	]
}
```

### Instantiate

```shell
CODE_ID='1876'
INIT='{"pool_id":1, "lock_duration": 0}'
osmosisd tx wasm instantiate $CODE_ID $INIT \
 --from wallet --label "vault" --gas-prices 0.025uosmo --gas auto --gas-adjustment 1.3 -b block -y --no-admin
```

### Execute

Deposit

```shell
DEPOSIT='{"deposit": {}}'
AMOUNT='10000000uosmo' # 10 OSMO
osmosisd tx wasm execute $CONTRACT_ADDR $DEPOSIT --from wallet --gas-prices 0.025uosmo --gas auto --gas-adjustment 1.3 --amount $AMOUNT -y
```

Compound

```shell
COMPOUND='{"compound": { "min_shares": 1 }}'
osmosisd tx wasm execute $CONTRACT_ADDR $COMPOUND --from wallet --gas-prices 0.025uosmo --gas auto --gas-adjustment 1.3 -y
```

### Query

```shell
QUERY='{"query_pool_request": {"pool_id": 1}}'
osmosisd query wasm contract-state smart $CONTRACT_ADDR $QUERY --output json
```

## References

- https://docs.cosmwasm.com/docs/1.0/
- https://docs.osmosis.zone/cosmwasm/testnet/cosmwasm-deployment
- https://docs.osmosis.zone/osmosis-core/modules/spec-gamm
- https://github.com/CosmWasm/rust-optimizer
- https://github.com/osmosis-labs/osmosis-rust/tree/main/packages/osmosis-std
- https://github.com/osmosis-labs/osmosis-rust/tree/main/examples/cosmwasm
- https://github.com/CosmWasm/cw-plus
- https://github.com/CosmWasm/cw-tokens/tree/main/contracts/cw20-staking
