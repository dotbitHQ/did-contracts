# das-contracts


## Development

### How to compile

To compile scripts on your own is the first step to start developing scripts, but it always cause headache problems when we facing problems about dependencies. 
so here we choose the simple solution provided by Nervos team, the Docker. With an image named `jjy0/ckb-capsule-recipe-rust` which contains all dependencies 
for compiling scripts, anyone may start compiling in about half an hour and the most time cost will be waiting for downloading. Thanks Nervos team! 👍

> ⚠️ Linux is recommended system to do the compiling task, otherwise you may face a little bit of performance issues of docker.

- Install docker base on [official documentation](https://docs.docker.com/engine/install/);
- Pull the compiling image with `docker pull jjy0/ckb-capsule-recipe-rust:2020-9-28`;
- Install `build-essential` if you are using Linux, install `xcode-select --install` if you are using MacOS;
- Install `pkg-config libssl-dev` to make openssl available as dependency if you are using Ubuntu, other Linux distributions may also need to do something similar.

Now you can start compiling scripts by yourself! 🚀

### Compiling commands

First you need to start container with `./docker.sh start`, then you can try commands below:

- `./docker.sh build xxx-cell-type --dev` compiling a specific script for development environment, it can be also `--local, --testnet2, --testnet3, --mainnet`;
- `./docker.sh build-all --dev` compiling all scripts for development environment;
- `./docker.sh build xxx-cell-type --release --dev` compiling a specific script with release profile, but still for development environment;
- `./docker.sh build-all --release --dev` compiling all scripts with release profile, but still for development environment;

> ⚠️ `./docker.sh` is a very simple script which can not handle arguments in different order, so remember keep all arguments as the same order as above otherwise it may not working properly.

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

### BE CAREFUL!

- DO NOT use `ckb_types::bytes`, IDE may treat it as `bytes-v0.5.6`, but it is `molecule::bytes` indeed, that is just a simple wrapper for `Vec<u8>`.
- DO NOT use `bytes-v0.5.6`, it will cause `VM Internal Error: InvalidInstruction(335951151)` for some reasons.


## Documents

- CKB VM Error Codes: https://github.com/nervosnetwork/ckb-system-scripts/wiki/Error-codes
