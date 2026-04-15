# AGENTS.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Core commands

### Build and run
- Build workspace (release): `cargo build --release`
- Build optimized node binary: `cargo build --release -p aether-node`
- Run local dev chain from source: `./target/release/aether-node --dev`
- Run compiled binary help: `./target/release/aether-node -h`
- Purge local dev chain data: `./target/release/aether-node purge-chain --dev`

### Test
- Run all tests in workspace: `cargo test --workspace`
- Run tests for the template pallet only: `cargo test -p aether-pallet-template`
- Run a single test by name (exact match): `cargo test -p aether-pallet-template it_works_for_default_value -- --exact`

### Lint/format
- Format all crates: `cargo fmt --all`
- Check formatting without writing: `cargo fmt --all -- --check`
- Lint all targets in workspace: `cargo clippy --workspace --all-targets -- -D warnings`

### Benchmarks and runtime-featured builds
- Build node with runtime benchmarks enabled: `cargo build -p aether-node --features runtime-benchmarks`
- Run benchmark subcommands via node CLI: `cargo run -p aether-node -- benchmark --help`
- Build with try-runtime feature: `cargo build -p aether-node --features try-runtime`

## Repository architecture

This is a Substrate-based Rust workspace with three primary crates declared in the workspace root:
- `node`: executable and service wiring (`aether-node`)
- `runtime`: FRAME runtime (`aether-runtime`)
- `pallets/template`: custom pallet scaffold (`aether-pallet-template`)

### Execution path (node side)
- `node/src/main.rs` delegates CLI handling to `command::run()`.
- `node/src/command.rs` maps CLI subcommands to either:
  - chain-spec loading and utility commands, or
  - full node startup through `service::new_full`.
- `node/src/service.rs` is the node assembly point:
  - creates client/backend/transaction pool/import queue in `new_partial`,
  - wires networking and sync (`build_network`, GRANDPA warp sync),
  - starts consensus tasks (Aura authoring + GRANDPA finality),
  - injects RPC extensions from `node/src/rpc.rs`,
  - optionally enables offchain workers.

When changing node behavior, most cross-cutting changes land in `service.rs`; command-surface changes land in `cli.rs`/`command.rs`.

### Chain spec and genesis
- `node/src/chain_spec.rs` defines `dev` and `local` chain specs and sets token symbol to `AETH`.
- Runtime genesis presets are implemented in `runtime/src/genesis_config_presets.rs` and referenced by chain-spec preset names.
- `customSpec.json` is a concrete chain spec artifact that can be used for custom launches.

### Runtime composition and behavior
- `runtime/src/lib.rs` defines runtime types, constants, transaction extension pipeline, and pallet composition via `#[frame_support::runtime]`.
- Pallets currently composed include System, Timestamp, Aura, Grandpa, Balances, TransactionPayment, Sudo, Treasury, and Template.
- Most pallet parameterization/config lives in `runtime/src/configs/mod.rs`.
  - Notable behavior: transaction fees and tips are redirected to Treasury via `runtime/src/configs/fee_handler.rs`.
- Runtime API exposure and host<->runtime integration are in `runtime/src/apis.rs` (metadata, block building, tx validation, session keys, payment APIs, benchmarks, try-runtime hooks, genesis builder APIs).

### Custom pallet area
- `pallets/template/src/lib.rs` is the local pallet scaffold and is included in the runtime as `Template`.
- Unit tests for the pallet live in `pallets/template/src/tests.rs` using a mock runtime (`mock.rs`).
- If adding business logic, this pallet (or additional pallets under `pallets/`) is the primary extension point; then wire config and inclusion in `runtime/src/lib.rs` + `runtime/src/configs/mod.rs`.

## Toolchain expectations
- `env-setup/rust-toolchain.toml` specifies stable toolchain components plus `wasm32-unknown-unknown` target.
- `docs/rust-setup.md` notes Substrate’s Unix-first setup and WSL recommendation for Windows development.
