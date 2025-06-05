# Mastic

![mastic logo](./assets/images/logo.png)

[![license-mit](https://img.shields.io/badge/License-MIT-teal.svg)](https://opensource.org/license/mit/)
[![ci state](https://github.com/veeso/mastic/actions/workflows/ci.yml/badge.svg)](https://github.com/veeso/mastic/actions/workflows/ci.yml)
[![Conventional Commits](https://img.shields.io/badge/Conventional%20Commits-1.0.0-%23FE5196?logo=conventionalcommits&logoColor=white)](https://conventionalcommits.org)
![icp](https://img.shields.io/badge/Internet%20Computer-FF5000?logo=InternetComputer)

Mastic aims to bring the Fediverse - a decentralized network of interconnected social platforms - natively onto the Internet Computer ecosystem.

## Get Started

### Prerequisites

- [Rust (1.85 or later)](https://rustup.rs/): to build the canisters
- [DFX](https://internetcomputer.org/docs/building-apps/getting-started/install) (v0.23 or later)
- [Just](https://just.systems/) to run scripts
- [ic-wasm](https://github.com/dfinity/ic-wasm): to bundle the canisters
- [candid-extractor](https://github.com/dfinity/candid-extractor): to extract the candid interface of the canisters

### Build canisters

Just run the following command to build all canisters:

```sh
just build_all_canisters
```

### Test canisters

To run the tests, run the following command:

```sh
just test [test_name]
just integration_test [test_name]
```

### Lint and format

```sh
just clippy
just fmt_nightly
```

## License

This project is licensed under the [MIT License](LICENSE).
