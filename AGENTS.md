# Repository Guidelines

## Project Structure & Module Organization

This is a small Rust DDNS client. Source lives in `src/`: `main.rs` owns CLI parsing and process flow, `lib.rs` exposes library modules, `ip.rs` discovers public IPv4/IPv6 addresses, `update.rs` defines provider-neutral update types, and `cloudflare.rs` implements Cloudflare API calls. Tests currently live beside the code they cover in `#[cfg(test)] mod tests` blocks. Add integration tests under `tests/` only when exercising the compiled CLI or cross-module behavior that cannot be covered cleanly with module tests. Nix packaging and development shell definitions live in `flake.nix` and `default.nix`.

## Build, Test, and Development Commands

Use the Nix shell when possible:

```sh
nix develop
cargo build
cargo test
cargo fmt
cargo clippy --all-targets --all-features
```

`cargo build` compiles the binary and library. `cargo test` runs unit and integration tests. `cargo fmt` applies the repository formatter. `cargo clippy --all-targets --all-features` catches common Rust mistakes across binaries and tests. To run locally against Cloudflare, use:

```sh
cargo run -- cloudflare --api-token "$CLOUDFLARE_API_TOKEN" --record-name host.example.com
```

Add `--allow-create` only when missing records should be created.

## Coding Style & Naming Conventions

Use Rust 2024 idioms and standard `rustfmt` formatting. Prefer small modules with explicit ownership of behavior: CLI in `main.rs`, IP discovery in `ip.rs`, provider update orchestration in `update.rs`, and provider API details in provider-specific files. Use `snake_case` for functions, variables, and modules; `PascalCase` for structs and enums; and concise enum variants such as `Fatal` or `Retryable`. Keep logging actionable and avoid printing secrets.

## Testing Guidelines

Keep tests deterministic and offline. Do not call Cloudflare or `ifconfig.me` from tests; instead isolate pure parsing, IP eligibility, zone selection, request-body construction, CLI parsing, and update error classification. If network-facing behavior needs coverage, introduce a small seam around the HTTP client or use a local mock server in an integration test.

Current test placement:

- `src/ip.rs`: web response parsing and public-address eligibility for IPv4/IPv6.
- `src/cloudflare.rs`: most-specific zone selection plus Cloudflare A/AAAA update and create request bodies.
- `src/update.rs`: second-level domain extraction and provider-neutral request construction.
- `src/main.rs`: clap parsing for `--4`, `--6`, Cloudflare required arguments, and `--allow-create`.
- `src/lib.rs`: public module entry points and public request constructors.

Prefer behavior names such as `selects_most_specific_zone_for_nested_record`, `rejects_cloudflare_without_api_token`, or `builds_ipv4_create_request_body_with_proxied_disabled`. For new provider code, cover record lookup/create/update decision logic without requiring credentials. For CLI changes, test `Cli::try_parse_from` in `main.rs` unless the behavior depends on process exit codes or stdout/stderr, in which case add an integration test under `tests/`.

Run `cargo test` before handing off changes. Run `cargo clippy --all-targets --all-features` for changes that touch shared update or provider code, and run `cargo fmt` before committing.

## Commit & Pull Request Guidelines

Recent history uses short Conventional Commit-style messages: `feat(ip): add web fetching`, `fix(nix): update cargo hash`, `chore(logging): add flags`. Follow `type(scope): summary` when a clear scope exists. Pull requests should include a brief behavior summary, test results, any new configuration requirements, and linked issues when applicable.

## Security & Configuration Tips

Never commit Cloudflare tokens or local output files. The repository already ignores `cloudflare_token_DO_NOT_COMMIT.txt`, `out.txt`, `target/`, `result/`, and `rust-toolchain`. Prefer environment variables for secrets, and keep debug logs free of API tokens and sensitive DNS account details.
