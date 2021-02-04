# das-contracts


## Development

Build specific:

```sh
./docker.sh build config-cell-type
```

Run tests:

``` sh
cargo test -p tests -- --nocapture
```

> Do not use `capsule build` and `capsule test` for performance reasons.

### Recompile Flow

1. Build config-cell-type.
2. Calculate code_hash of config-cell-type.


## Documents

- CKB VM Error Codes: https://github.com/nervosnetwork/ckb-system-scripts/wiki/Error-codes
