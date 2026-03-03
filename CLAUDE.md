# Mastic

Mastic is a federated social platform (Mastodon-compatible via ActivityPub) running entirely on the Internet Computer as Rust WASM canisters.

## Project Structure

```
crates/
  did/          # Shared Candid types library (no cdylib)
  directory/    # Directory Canister — user discovery, handle→canister mapping
  federation/   # Federation Canister — S2S HTTP, WebFinger, ActivityPub
  user/         # User Canister — per-user actor, inbox/outbox, Social API
integration-tests/
  pocket-ic-tests/        # Integration tests using pocket-ic
  pocket-ic-tests-macro/  # Proc-macro support for integration tests
docs/
  activitypub/  # ActivityPub protocol reference and Mastic mapping
  architecture/ # Architecture overview and sequence diagrams
  interface/    # Canonical Candid .did files (directory, federation, user)
  project.md    # Project spec, user stories, milestones, interface definitions
```

## Build & Test

All commands use [just](https://just.systems/). Run `just` to list available commands.

```sh
just build_all_canisters    # Build all three canisters (directory, federation, user)
just build_directory        # Build only the directory canister
just build_federation       # Build only the federation canister
just build_user             # Build only the user canister
just test                   # Run unit tests
just integration_test       # Run pocket-ic integration tests
just test_all               # Run all tests (unit + integration)
just check_code             # Run nightly rustfmt --check + clippy -D warnings
just fmt                    # Format code (stable)
just fmt_nightly            # Format code (nightly, used in CI)
```

Build pipeline: `cargo build --target wasm32-unknown-unknown` → `ic-wasm shrink` → `candid-extractor` → `gzip`. Artifacts go to `.artifact/`.

## Code Quality

- **Formatting**: Uses nightly rustfmt. Config in `rustfmt.toml` (imports grouped by `StdExternalCrate`, module-level granularity).
- **Linting**: `cargo clippy --all-features --all-targets -- -D warnings` — zero warnings policy.
- **CI** runs `just check_code` then `just build_all_canisters` then `just test_all`.

## Architecture

Three canister types, each a separate workspace crate:

- **Directory Canister**: Global registry. Maps Internet Identity principals to handles and User Canister IDs. Manages sign-up, profile deletion, moderation (suspend, add/remove moderator).
- **Federation Canister**: HTTP boundary. Handles all S2S ActivityPub traffic (incoming via `http_request`/`http_request_update`, outgoing via `send_activity`). Serves WebFinger, actor profiles, collections. Routes activities between local User Canisters via the Directory Canister.
- **User Canister**: One per user. Stores inbox, outbox, profile, followers, following, liked collections. Exposes typed Candid methods as the Social API (C2S replacement). Holds RSA keypair for HTTP Signatures.
- **did**: Shared library crate for Candid type definitions used across canisters.

Authorization is principal-based: User→UserCanister (owner principal), Federation→UserCanister (federation principal in install args), UserCanister→Federation (registered at creation).

## Conventions

### Git

- **Conventional commits**: `feat:`, `fix:`, `docs:`, `chore:`, `test:`, `refactor:`, etc.
- **Never** add `Co-Authored-By` lines to commits.

### Rust

- Edition 2024, resolver 3, minimum Rust version 1.90.0 (toolchain pinned to 1.93.0).
- Target: `wasm32-unknown-unknown` for canisters.
- Workspace dependencies defined in root `Cargo.toml` — crates reference them with `workspace = true`.
- Key dependencies: `ic-cdk` 0.19, `candid` 0.10, `ic-stable-structures` 0.7, `pocket-ic` 12.

### Candid Interfaces

Canonical `.did` files live in `docs/interface/`. The build pipeline also auto-extracts `.did` from WASM to `.artifact/`. When adding or modifying canister methods, update **both**:
1. `docs/interface/{canister}.did` — the spec
2. `docs/project.md` — the Interface section (must match the .did files exactly)

### Documentation

- `docs/project.md` is the single source of truth for user stories, milestones, and interface specs.
- `docs/architecture/README.md` contains sequence diagrams for all major flows.
- `docs/activitypub/README.md` is the ActivityPub protocol reference with Mastic-specific mapping.
- When adding a new flow, add both a sequence diagram in architecture and a user story in project.md.

## Local Development

```sh
just dfx_start              # Start local DFX replica
just dfx_deploy_local       # Deploy all canisters locally
```

Requires: `dfx` >= 0.30.2, `ic-wasm`, `candid-extractor`, Rust nightly (for formatting).
