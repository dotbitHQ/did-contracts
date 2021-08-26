# das-contracts

![Version](https://img.shields.io/github/release/DeAccountSystems/das-contracts.svg)
![License](https://img.shields.io/github/license/DeAccountSystems/das-contracts.svg)
[![Telegram](https://img.shields.io/badge/Telegram-2CA5E0?style=for-the-badge&logo=telegram&logoColor=white)](https://t.me/DASystemsNews)
[![Discord](https://img.shields.io/badge/Discord-7289DA?style=for-the-badge&logo=discord&logoColor=white)](https://discord.gg/WVunwT2hju)
[![Twitter](https://img.shields.io/badge/Twitter-1DA1F2?style=for-the-badge&logo=twitter&logoColor=white)](https://twitter.com/realDASystems)

This repository is open source for DAS contracts which also called "type script" in CKB. They can only execute in 
[ckb-vm](https://github.com/nervosnetwork/ckb-vm) environment which a pure software implementation of the RISC-V 
instruction set.


## About DAS

DAS is a blockchain-based, open source, censorship-resistant decentralized account system that provides a globally unique naming system with a .bit suffix that can be used for cryptocurrency transfers, domain name resolution, authentication, and other scenarios.

Now DAS has been deployed on CKB mainnet which named **Lina** and launched from **2021-07-22** 🎉 .

## Development

### Prerequisites

Make sure you have installed all the following prerequisites on your development machine:
- Docker - [Install Docker](https://docs.docker.com/engine/install/).
- Rust - [Install Rust](https://www.rust-lang.org/tools/install) and [switch to nightly version](https://rust-lang.github.io/rustup/concepts/channels.html) using the following commands:
```shell
rustup toolchain install nightly # install nightly
rustup default nightly # use nightly
``` 
- Install `build-essential` if you are using Linux, install `xcode-select --install` if you are using macOS;
- Install `pkg-config libssl-dev` to make openssl available as dependency if you are using Ubuntu, other Linux distributions may also need to do something similar.


### How to compile

To compile scripts on your own is the first step to start developing scripts, but it always causes headache problems when we're facing problems about dependencies. 
So here we choose a simple solution provided by Nervos team - using Docker. With an image named `jjy0/ckb-capsule-recipe-rust` which contains all dependencies 
for compiling scripts, anyone can start compiling in about half an hour, and the most time cost will be waiting for downloading. Thanks Nervos team! 👍

> ⚠️ Linux is recommended system to do the compiling task, otherwise you may face a little bit of performance issues of docker.

- Pull the compiling image with `docker pull jjy0/ckb-capsule-recipe-rust:2020-9-28`;

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
cargo test -p tests gen_
cargo test -p tests test_
cargo test -p tests challenge_
```

> DO NOT use `capsule build` and `capsule test` for performance reasons.

### BE CAREFUL!

- DO NOT use `ckb_types::bytes`, IDE may treat it as `bytes-v0.5.6`, but it is `molecule::bytes` indeed, that is just a simple wrapper for `Vec<u8>`.
- DO NOT use `bytes-v0.5.6`, it will cause `VM Internal Error: InvalidInstruction(335951151)` for some reasons.

### Documents

- For details about price, preserved accounts and so on, please see: https://docs.da.systems/docs/v/English/
- To learn more about data structures, protocols and other technical details, please see documents in [docs/](docs) directory of this repository.
- It's a good idea to start with their RFCs to learn more about all aspects of CKB: https://github.com/nervosnetwork/rfcs
- Other things may help you a lot when develop contracts:
  - CKB VM Error Codes: https://github.com/nervosnetwork/ckb-system-scripts/wiki/Error-codes
  - CKB JSON-RPC Protocols: https://github.com/nervosnetwork/ckb/tree/develop/rpc


## License

This repository is released under the terms of the MIT license. See [LICENSE](LICENSE) for more information or see https://choosealicense.com/licenses/mit/.
