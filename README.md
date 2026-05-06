# Rusty DDNS

A small dynamic DNS client written in Rust. It discovers the current public IPv4
and/or IPv6 address for the host and updates DNS records through the selected
provider.

## Supported Platforms

- Cloudflare

## Installation

### Option 1: Use as a Nix flake

Run the package directly:

```sh
nix run github:liam-murphy14/rusty_ddns -- \
  cloudflare \
  --api-token "$CLOUDFLARE_API_TOKEN" \
  --record-name host.example.com
```

Install it into your user profile:

```sh
nix profile install github:liam-murphy14/rusty_ddns
```

Then run:

```sh
rusty_ddns cloudflare \
  --api-token "$CLOUDFLARE_API_TOKEN" \
  --record-name host.example.com
```

The flake currently builds for:

- `x86_64-linux`
- `aarch64-linux`
- `x86_64-darwin`
- `aarch64-darwin`

### Option 2: Install the binary from GitHub

Install directly from the repository with Cargo:

```sh
cargo install --git https://github.com/liam-murphy14/rusty_ddns.git
```

Then run:

```sh
rusty_ddns cloudflare \
  --api-token "$CLOUDFLARE_API_TOKEN" \
  --record-name host.example.com
```

## Usage

Set a Cloudflare API token in your environment:

```sh
export CLOUDFLARE_API_TOKEN="your-token"
```

Update both A and AAAA records when public IPv4 and IPv6 addresses are
available:

```sh
rusty_ddns cloudflare \
  --api-token "$CLOUDFLARE_API_TOKEN" \
  --record-name host.example.com
```

Update only the A record:

```sh
rusty_ddns --4 cloudflare \
  --api-token "$CLOUDFLARE_API_TOKEN" \
  --record-name host.example.com
```

Update only the AAAA record:

```sh
rusty_ddns --6 cloudflare \
  --api-token "$CLOUDFLARE_API_TOKEN" \
  --record-name host.example.com
```

Create missing records when needed:

```sh
rusty_ddns cloudflare \
  --api-token "$CLOUDFLARE_API_TOKEN" \
  --record-name host.example.com \
  --allow-create
```

Increase logging verbosity with clap verbosity flags, for example:

```sh
rusty_ddns -v cloudflare \
  --api-token "$CLOUDFLARE_API_TOKEN" \
  --record-name host.example.com
```

The Cloudflare token must have permission to read zones and edit DNS records for
the target zone.

Example systemd service and timer units are available in `examples/` for running
the Cloudflare update every 5 minutes.

## Contributing

Keep changes focused and covered by deterministic, offline tests. Do not call
Cloudflare or public IP lookup services from tests.

### Nix workflow

Enter the development shell:

```sh
nix develop
```

Build, test, format, and lint:

```sh
cargo build
cargo test
cargo fmt
cargo clippy --all-targets --all-features
```

Format Nix files:

```sh
nix fmt
```

### Pure Rust workflow

Install a current stable Rust toolchain, then run:

```sh
cargo build
cargo test
cargo fmt
cargo clippy --all-targets --all-features
```

For provider changes, cover request construction, record lookup/update/create
decision logic, and error classification without requiring credentials. For CLI
changes, prefer parser tests around `Cli::try_parse_from` unless the behavior
depends on process exit codes or stdout/stderr.

Never commit API tokens, local output files, or generated build artifacts.
