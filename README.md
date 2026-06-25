# Cerebrum

A two-tier agent memory subsystem implemented as a single Model Context Protocol (MCP) server.

## Quick Start

```bash
nix develop . --command just run
```

## Development

```bash
# Format code
nix develop . --command just fmt

# Run linter
nix develop . --command just lint

# Run tests
nix develop . --command just test

# Check code coverage (must be ≥90%)
nix develop . --command just coverage
```

## Code Quality Requirements

- **Coverage Gate:** All code must maintain ≥90% test coverage (enforced by `just coverage`)
- **Formatting:** Code must be formatted with `cargo fmt`
- **Linting:** All clippy warnings must be fixed (run `just lint`)

## Architecture

See [docs/architecture.md](docs/architecture.md) for system design and memory tier documentation.

## License

MIT
