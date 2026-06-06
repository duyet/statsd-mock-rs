# Contributing to statsd-mock-rs

Thank you for your interest in contributing! This document provides guidelines and instructions for contributing to statsd-mock-rs.

## Code of Conduct

Be respectful, professional, and constructive. We're all here to make this project better.

## Before You Start

1. **Check existing issues** - Someone might already be working on it
2. **Open an issue first** - Discuss your idea before implementing
3. **Read CLAUDE.md** - Understand the project philosophy and architecture

## Development Setup

### Prerequisites

- Rust 1.62.0 or later (MSRV)
- Git

### Clone and Build

```bash
git clone https://github.com/duyet/statsd-mock-rs.git
cd statsd-mock-rs
cargo build
cargo test
```

### Check Your Code

Before submitting, ensure:

```bash
# All tests pass
cargo test

# Code is formatted
cargo fmt --all -- --check

# No clippy warnings
cargo clippy -- -D warnings

# Works with MSRV
cargo +1.62.0 check
```

## Making Changes

### 1. Branch Naming

Use descriptive branch names:
- `feat/add-xyz` - New features
- `fix/issue-123` - Bug fixes
- `docs/improve-readme` - Documentation
- `refactor/cleanup-xyz` - Refactoring

### 2. Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add histogram lookup method
fix: resolve race condition in packet collection
docs: update README with new API examples
test: add tests for edge cases
chore: update dependencies
```

### 3. Code Style

- Follow Rust naming conventions
- Add doc comments to public APIs
- Include examples in doc comments
- Mark query methods with `#[must_use]`
- Keep functions focused and small

### 4. Testing Requirements

Every change must include tests:

- **New features**: Add happy path + error cases
- **Bug fixes**: Add regression test
- **Refactoring**: Ensure existing tests still pass

Minimum test coverage per public method:
1. Happy path test
2. Error case test
3. Integration test (if applicable)

Example:
```rust
#[test]
fn test_feature_happy_path() {
    // Test normal usage
}

#[test]
fn test_feature_error_case() {
    // Test invalid input
}

#[test]
fn test_feature_integration() {
    // Test with real statsd crate
}
```

### 5. Documentation

Add or update:
- Doc comments on public APIs
- README examples (if API changes)
- CLAUDE.md (for architectural changes)

## Pull Request Process

### 1. Before Submitting

Checklist:
- [ ] Tests pass: `cargo test`
- [ ] Formatted: `cargo fmt`
- [ ] No warnings: `cargo clippy`
- [ ] MSRV compatible: `cargo +1.62.0 check`
- [ ] Documentation updated
- [ ] Examples updated (if needed)

### 2. PR Description

Include:
- **What**: Brief description of changes
- **Why**: Reason for the change
- **How**: Implementation approach
- **Testing**: How you tested it
- **Breaking Changes**: If any (avoid if possible)

Template:
```markdown
## What

Add `histogram()` lookup method to CapturedPackets

## Why

Users need to lookup histogram values by name, similar to counter/gauge

## How

- Added `histogram()` method returning `Option<f64>`
- Added `assert_histogram()` fluent assertion
- Added tests covering happy path and error cases

## Testing

- Added `test_histogram_lookup()`
- Added `test_histogram_not_found()`
- Added `test_assert_histogram()`

## Breaking Changes

None - all changes are additive
```

### 3. Review Process

1. Automated checks run (CI)
2. Maintainer reviews code
3. Address feedback
4. Once approved, maintainer merges

## Contribution Ideas

Good first contributions:

### Easy
- Add more examples
- Improve documentation
- Add doc tests
- Fix typos

### Medium
- Add new assertion methods
- Add collection helpers
- Improve error messages
- Add benchmarks

### Advanced
- Add new metric type support
- Optimize packet parsing
- Add async support
- Improve timing algorithm

See [Issues](https://github.com/duyet/statsd-mock-rs/issues) for specific tasks.

## Architecture Guidelines

From CLAUDE.md:

### Core Principles

1. **Simplicity First** - This is a testing utility, not a production server
2. **Zero Configuration** - Works out of the box
3. **Backward Compatibility** - Never break existing code
4. **Type Safety** - Compile-time guarantees over runtime checks
5. **Elegant APIs** - Code should read like prose

### Design Constraints

- **MSRV**: Rust 1.62.0 (don't raise without discussion)
- **Dependencies**: Minimal (justify new dependencies)
- **Single Purpose**: Mock StatsD packets for testing only
- **In-Process**: No external services

### API Patterns

- **Consuming self**: `capture(self, ...)` prevents reuse
- **Fluent assertions**: Return `Self` for chaining
- **Type-safe lookups**: Return correct types immediately
- **`#[must_use]`**: On all query methods

## Release Process

(For maintainers)

1. Update version in `Cargo.toml`
2. Update README if needed
3. Commit: `git commit -m "chore: release v0.x.y"`
4. Tag: `git tag v0.x.y`
5. Push: `git push --tags`
6. CI publishes to crates.io automatically

## Questions?

- **Technical questions**: Open a GitHub discussion
- **Bug reports**: Open an issue
- **Security issues**: Email me@duyet.net privately
- **General chat**: Open a discussion

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to statsd-mock-rs! 🎉
