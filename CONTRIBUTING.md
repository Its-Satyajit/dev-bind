# Contributing to DevBind

Thank you for your interest in contributing to DevBind!

## Development Setup

```bash
git clone https://github.com/Its-Satyajit/dev-bind.git
cd dev-bind
cargo build
```

## Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy -- -D warnings` to check for issues
- Follow existing code conventions in the codebase

## Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/your-feature`)
3. Make your changes and ensure tests pass
4. Push to your fork and submit a pull request
5. Ensure PR description explains the changes and motivation

## Testing

Run tests with:
```bash
cargo test
```

## Building

```bash
# Full build (CLI + GUI)
cargo build --release

# CLI only
cargo build --release --no-default-features --features cli
```

## Questions

For questions, open a GitHub discussion or issue.
