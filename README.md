# Mastic

![Mastic logo](./assets/images/logo-150.png)

Unleashing the power of IC on the Fediverse

[![license-mit](https://img.shields.io/badge/License-MIT-teal.svg)](https://opensource.org/license/mit/)
[![Conventional Commits](https://img.shields.io/badge/Conventional%20Commits-1.0.0-%23FE5196?logo=conventionalcommits&logoColor=white)](https://conventionalcommits.org)
![icp](https://img.shields.io/badge/Internet%20Computer-FF5000?logo=InternetComputer)

[![CI](https://img.shields.io/github/actions/workflow/status/veeso/mastic/ci.yml?branch=main)](https://github.com/veeso/mastic/actions/workflows/ci.yml)
[![Coverage Status](https://coveralls.io/repos/github/veeso/mastic/badge.svg?branch=main)](https://coveralls.io/github/veeso/mastic?branch=main)

[Documentation](https://docs.mastic.social)

---

Mastic is a federated social platform (Mastodon-compatible via ActivityPub) running entirely
on the [Internet Computer](https://internetcomputer.org/) as Rust WASM canisters.

Everyone can run their own Mastic instance and interact seamlessly with the wider Fediverse,
while benefiting from the Internet Computer's scalability, Internet Identity authentication,
and decentralised governance through a DAO (SNS).

## Architecture

Mastic consists of three canister types plus shared libraries:

| Component | Description |
| --------- | ----------- |
| **Directory Canister** | Global registry. Maps Internet Identity principals to handles and User Canister IDs. Manages sign-up, profile deletion, and moderation. |
| **Federation Canister** | HTTP boundary. Handles all server-to-server ActivityPub traffic, serves WebFinger, and routes activities between local User Canisters and the Fediverse. |
| **User Canister** | One per user. Stores inbox, outbox, profile, followers, following, and liked collections. Holds an RSA keypair for HTTP Signatures. |
| **activitypub** (lib) | ActivityPub protocol types and utilities. |
| **did** (lib) | Shared Candid type definitions used across canisters. |

```text
crates/
  canisters/
    directory/
    federation/
    user/
  libs/
    activitypub/
    db-utils/
    did/
    ic-utils/
integration-tests/
  pocket-ic-tests/
  pocket-ic-tests-macro/
docs/                        # mdBook documentation
```

Authorization is principal-based: User → User Canister (owner principal),
Federation → User Canister (federation principal in install args),
User Canister → Federation (registered at creation).

For detailed architecture diagrams and protocol flows, see the
[architecture documentation](https://docs.mastic.social/architecture.html).

## Get Started

### Prerequisites

- [Rust](https://rustup.rs/) (1.90.0 or later)
- [DFX](https://internetcomputer.org/docs/building-apps/getting-started/install) (v0.31.0 or later)
- [Just](https://just.systems/)
- [ic-wasm](https://github.com/dfinity/ic-wasm)
- [candid-extractor](https://github.com/dfinity/candid-extractor)

### Build

```sh
just build_all    # Build all three canisters
just build_directory        # Build only the directory canister
just build_federation       # Build only the federation canister
just build_user             # Build only the user canister
```

Build pipeline: `cargo build --target wasm32-unknown-unknown` → `ic-wasm shrink`
→ `candid-extractor` → `gzip`. Artifacts go to `.artifact/`.

### Test

```sh
just test                   # Unit tests
just integration_test       # pocket-ic integration tests
just test_all               # All tests (unit + integration)
```

### Lint and Format

```sh
just check_code             # nightly rustfmt --check + clippy -D warnings
just fmt                    # Format (stable)
just fmt_nightly            # Format (nightly, used in CI)
```

### Local Deployment

```sh
just dfx_start              # Start local DFX replica
just dfx_deploy_local       # Deploy all canisters locally
```

## Contributing

Contributions are welcome. Please follow [conventional commits](https://conventionalcommits.org)
for commit messages.

CI runs `just check_code` → `just build_all` → `just test_all`. Make sure all three
pass before submitting a PR.

## License

This project is licensed under the [MIT License](LICENSE).
