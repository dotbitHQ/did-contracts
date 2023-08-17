# DAS Serialization Reference

DAS choose [Molecule][1] as data serialization standard, this serialization system is maintaining by [Nervos](https://nervos.org).

## `std` and `non_std`

> WARNING, read this first otherwise you will face memory issues when compiling.

Since some versions of ckb_std it can no more be used with std environment, so we have to split this library into two branches. One support 
non_std environment, one support std environment. For non_std version you may keep using the `develop` branch, and for std version you must 
using the `develop-std` branch from now on.

And the name of the crate has been renamed to **das-types-std**, because [das-contracts]() depends on the non_std branch in contracts and 
std branch in unit tests at the same time, so they can not share the same name otherwise cargo will report ambiguous dependency error.

## Development

### Setup environment

The only thing need here is installing [Molecule][1] and language plugin:

```shell
cargo install moleculec moleculec-go
go install github.com/xxuejie/moleculec-es/cmd/moleculec-es@latest
```

For more details please read [Molecule][1].

### Generate schema codes

Simply run `sh compile.sh <language>`. Currently the following language is supported:

- Rust;
- Go;
- JavaScript/TypeScript;

> Language name should be lower case in CLI. If your language is not yet supported here, feel free to submit a PR. 😅

[1]: https://github.com/nervosnetwork/molecule
