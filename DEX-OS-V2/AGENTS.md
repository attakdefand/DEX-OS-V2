# Repository Guidelines

## Project Structure & Module Organization
The workspace is defined in `Cargo.toml` and groups four Rust crates: `dex-core/` for matching and AMM logic, `dex-api/` for the HTTP layer, `dex-db/` for persistence helpers, and `dex-wasm/` for browser bindings. Shared integration tests live in `tests/`, and build helpers such as `build-wasm.sh` and `dev-tools.sh` sit in the repository root. Treat `target/` as generated output and avoid committing its contents.

## Build, Test, and Development Commands
Use `cargo build` for a full workspace build and `cargo run -p dex-api` to launch the local API on `localhost:3030`. Run `cargo test` for the default suite or `cargo nextest run` when you require faster iteration. Build the WebAssembly package with `./build-wasm.sh` (or `build-wasm.bat` on Windows). The interactive helper `./dev-tools.sh` bundles the common build, lint, audit, and test tasks into one menu.

## Coding Style & Naming Conventions
All Rust code should remain `cargo fmt` clean and pass `cargo clippy -- -D warnings` before review. Follow standard Rust casing: modules and files in `snake_case`, types and traits in `PascalCase`, and constants in `SCREAMING_SNAKE_CASE`. Prefer explicit `use` statements grouped by crate, and document non-trivial modules with `//!` comments as in `tests/integration_test.rs`. Keep WASM exports aligned with the existing `wasm-bindgen` patterns inside `dex-wasm/`.

## Testing Guidelines
Add unit tests alongside implementation modules and integration tests under `tests/` using descriptive file names such as `order_matching_test.rs`. Tests should cover critical order-matching paths, AMM swaps, and database migrations similar to the existing integration suite. When new database migrations are introduced, assert their presence to match the checks in `tests/integration_test.rs`. Run `cargo test` (or `cargo nextest run`) before pushing and include reproduction steps for any flaky cases.

## Commit & Pull Request Guidelines
Recent commits use short, imperative messages such as “Implement StableSwap AMM for low slippage trades between pegged assets.” Follow that tone, capitalizing the first verb and omitting trailing punctuation. Each pull request should summarize the user-facing impact, link related issues, and call out breaking changes or schema additions. Include screenshots or `curl` transcripts when modifying API responses, and note any follow-up tasks required for deployment.

## Security & Operational Checks
Before merging, run `cargo audit` and `cargo deny check` (available through `./dev-tools.sh`) to surface dependency risks. Document any mitigations in the pull request, and never suppress advisories without recorded justification. For production-like testing, ensure secrets stay outside the repo and use environment variables loaded through your local `.env`.
