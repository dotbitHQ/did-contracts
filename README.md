# das-contracts


## Development

Build specific:

```sh
./docker.sh build config-cell-type
```

### Unit tests

All tests are divided into three categories:

- tests start with `gen_`, these tests are design for generating testing data for other tests.
- tests start with `test_`, these are basic tests for simply debugging contracts, there is no error in any of these tests.
- tests start with `challenge_`, these are boundary condition tests that a specific error code must be returned.

The prefix is design for running different categories of tests separately: 

``` sh
cargo test -p tests test_ -- --nocapture --test-threads=1
cargo test -p tests challenge_ -- --nocapture --test-threads=1
```

> Do not use `capsule build` and `capsule test` for performance reasons.

### Recompile Flow

1. Build config-cell-type.
2. Calculate code_hash of config-cell-type.


## Documents

- CKB VM Error Codes: https://github.com/nervosnetwork/ckb-system-scripts/wiki/Error-codes
