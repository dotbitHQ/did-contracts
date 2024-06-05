# did-contracts

![Version](https://img.shields.io/github/release/dotbitHQ/did-contracts.svg)
![License](https://img.shields.io/github/license/dotbitHQ/did-contracts.svg)
[![Discord](https://img.shields.io/badge/Discord-7289DA?style=for-the-badge&logo=discord&logoColor=white)](https://discord.com/invite/did)
[![Twitter](https://img.shields.io/badge/Twitter-1DA1F2?style=for-the-badge&logo=twitter&logoColor=white)](https://x.com/intent/follow?screen_name=DIDbased)

This repository is open source for DID contracts which also called "type script" in CKB. They can only execute in [ckb-vm](https://github.com/nervosnetwork/ckb-vm) environment which a pure software implementation of the RISC-V instruction set.


## About DID

DID is a blockchain-based, open source, censorship-resistant decentralized account system that provides a globally unique naming system with a .bit suffix that can be used for cryptocurrency transfers, domain name resolution, authentication, and other scenarios.

Now DID has been deployed on CKB mainnet which named **Lina** and launched from **2021-07-22** 🎉 .

## Development

### Prerequisites

Make sure you have installed all the following prerequisites on your development machine:
- Docker - [Install Docker](https://docs.docker.com/engine/install/).
- Rust - [Install Rust](https://www.rust-lang.org/tools/install) and its [nightly toolchain](https://rust-lang.github.io/rustup/concepts/toolchains.html)

- Install `build-essential` if you are using Linux, install `xcode-select --install` if you are using MacOS;
- Install `pkg-config libssl-dev` to make openssl available as dependency if you are using Ubuntu, other Linux distributions may also need to do something similar.


### How to compile

To compile scripts on your own is the first step to start developing scripts, but it always causes headache problems when we're facing problems about dependencies.
So here we choose a simple solution provided by Nervos team - using Docker. With an image named `dotbitteam/ckb-dev-all-in-one:0.0.1` which contains all dependencies
for compiling scripts, anyone can start compiling in about half an hour, and the most time cost will be waiting for downloading.

> ⚠️ Linux and x86 is recommended to do the compiling task, otherwise you may face various problems cause by both system and chipset.

- Pull the compiling image with `docker pull dotbitteam/ckb-dev-all-in-one:0.0.1`;

Now you can start compiling scripts by yourself! 🚀

### Compiling commands

First you need to start container with `./docker.sh start -b`, then you can try commands below:

- `./docker.sh build xxx-cell-type --dev` compiling a specific script for development environment, it can be
  also `--dev, --testnet, --mainnet`;
- `./docker.sh build-all --dev` compiling all scripts for development environment;
- `./docker.sh build xxx-cell-type --release --dev` compiling a specific script with release profile, but still for development environment;
- `./docker.sh build-all --release --dev` compiling all scripts with release profile, but still for development environment;

> ⚠️ `./docker.sh` is a very simple script which can not handle arguments in different order, so remember keep all arguments as the same
> order as above otherwise it may not work properly.

> When executing `build-all` sub-command, `test-env, test-custom-script` and `playground` scripts will not be compiled. You need to compile
> them one by one with `build` sub-command.

### Unit tests

All tests are divided into two categories:

- tests start with `test_`, these are basic tests for simply debugging contracts, there is no error in any of these tests.
- tests start with `challenge_`, these are boundary condition tests that a specific error code must be returned.

The prefix is design for running different categories of tests separately:

```bash
./docker.sh test-release test_ # all the tests start with test_ are normal tests
./docker.sh test-release challenge_ # all the tests start with challenge_ are abnormal tests
```

All tests will be executed with the above commands, but if any test fails, it is possible to get the detailed runtime log with the following command:

```bash
./docker.sh test-debug test_config_account_loading
```

### Documents

- For details about price, preserved accounts and so on, please see: https://community.d.id/c/knowledge-base-bit/
- To learn more about data structures, protocols and other technical details, please see documents in [docs/](https://github.com/dotbitHQ/did-contracts/tree/docs/docs) directory of this repository.
- It's a good idea to start with their RFCs to learn more about all aspects of CKB: https://github.com/nervosnetwork/rfcs
- Other things may help you a lot when develop contracts:
  - CKB VM Error Codes: https://github.com/nervosnetwork/ckb-system-scripts/wiki/Error-codes
  - CKB JSON-RPC Protocols: https://github.com/nervosnetwork/ckb/tree/develop/rpc


## Audit Report

The contracts were audited by [Least Authority](https://leastauthority.com/) on April 10, 2024. You can find the reports in the [**Audit Report - by Least Authority.pdf**](https://github.com/dotbitHQ/did-contracts/blob/master/Audit%20Report%20-%20by%20Least%20Authority.pdf) file.


## License

This repository is released under the terms of the MIT license. See [LICENSE](LICENSE) for more information or see https://choosealicense.com/licenses/mit/.
