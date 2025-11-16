# CLAUDE.md - Developer Guide for statsd-mock-rs

This document provides architectural guidance, design principles, and implementation details for maintaining and extending statsd-mock-rs.

## Project Philosophy

### Core Principles

1. **Simplicity First** - This is a testing utility, not a production StatsD server. Keep it focused.
2. **Zero Configuration** - Users should get value with `statsd_mock::start()` and nothing else.
3. **Backward Compatibility** - Never break existing code. All changes must be additive.
4. **Type Safety** - Prefer compile-time guarantees over runtime checks.
5. **Elegant APIs** - Code should read like prose, assertions should chain naturally.

### Design Constraints

- **Single Purpose**: Mock StatsD packets for testing - nothing more
- **No External Services**: Everything runs in-process
- **MSRV Stability**: Currently Rust 1.62.0 - avoid raising this unless absolutely necessary
- **Minimal Dependencies**: Only `itertools` for string joining - justify any additions
- **Test Coverage**: Every public method must have tests

## Architecture Overview

### Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                     StatsDServer                             │
│  - UDP socket on 127.0.0.1:0 (random port)                  │
│  - Background thread for packet collection                   │
│  - Intelligent timing (quiet period detection)               │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                  Packet Collection                           │
│  - recv_timeout loop with adaptive waiting                   │
│  - 50ms quiet period detection                               │
│  - 2s safety timeout                                         │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                    Packet Parsing                            │
│  - Parse raw UDP bytes to StatsD protocol                    │
│  - Type-safe Packet enum (Counter, Gauge, Timer, etc.)       │
│  - Graceful error handling with ParseError                   │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                  CapturedPackets                             │
│  - Collection of parsed packets with helper methods          │
│  - Fluent assertion API (.assert_counter(), etc.)            │
│  - Collection helpers (all_counters(), filter_by_name())     │
│  - Iterator support for natural traversal                    │
└─────────────────────────────────────────────────────────────┘
```

### Threading Model

```rust
Main Thread                          Background Thread
    │                                       │
    ├─ func()                              ├─ loop {
    │   └─ client.incr()                   │     recv_from(&mut buf)
    │       └─ UDP packet ────────────────►│     send(bytes)
    │                                       │     if func_ran.load() { break }
    ├─ [intelligent waiting]               │   }
    │   └─ recv_timeout(10ms)              │
    │   └─ quiet period check              │
    ├─ func_ran.store(true) ──────────────►│
    │                                       │
    └─ bg.join()                           └─ [exit]
```

### Intelligent Timing Strategy

**Problem**: Fixed `sleep(200ms)` was too slow OR too fast depending on environment.

**Solution**: Adaptive waiting based on packet arrival patterns.

```rust
const RECV_TIMEOUT_MS: u64 = 10;      // Poll every 10ms
const QUIET_PERIOD_MS: u64 = 50;      // 50ms silence = done
const MAX_WAIT_SECS: u64 = 2;         // Safety timeout

loop {
    match recv_timeout(10ms) {
        Ok(packet) => {
            last_packet_time = now();
            packets.push(packet);
        }
        Timeout => {
            if last_packet_time.elapsed() >= 50ms {
                break;  // Quiet period detected
            }
            if start.elapsed() >= 2s {
                break;  // Safety timeout
            }
        }
    }
}
```

**Benefits**:
- Fast machines: Exits as soon as packets stop (typically <100ms)
- Slow machines: Waits up to 2s without missing packets
- Deterministic: Based on actual network behavior, not arbitrary delays

## API Design Patterns

### 1. Consuming Self Pattern

```rust
pub fn capture<F>(self, func: F) -> String  // Note: self, not &self
```

**Rationale**: Each test gets a fresh server instance. Prevents accidental reuse and state pollution.

**Trade-off**: Can't reuse server, but this is intentional for test isolation.

### 2. Fluent Assertions

```rust
packets
    .assert_counter("name", 1)
    .assert_gauge("name", 42.0)
    .assert_timer("name", 100.0);
```

**Implementation**: All `assert_*` methods return `Self` for chaining.

**Benefits**: Readable test code, clear intent, composable assertions.

### 3. Type-Safe Lookups

```rust
pub fn counter(&self, name: &str) -> Option<i64>
pub fn gauge(&self, name: &str) -> Option<f64>
```

**Rationale**: Return correct type immediately, no string parsing in tests.

**Pattern**: Use `Option<T>` to handle missing metrics gracefully.

### 4. Collection Helpers

```rust
pub fn all_counters(&self) -> Vec<(&str, i64)>
pub fn filter_by_name(&self, name: &str) -> Vec<&Packet>
```

**Use Case**: Bulk operations, debugging, flexible test assertions.

## StatsD Protocol Support

### Packet Format

```
name:value|type[|@sample_rate][#tags]
```

### Supported Metric Types

| Type | Format | Example | Parsed As |
|------|--------|---------|-----------|
| Counter | `name:value\|c` | `requests:1\|c` | `Packet::Counter` |
| Gauge | `name:value\|g` | `memory:1024\|g` | `Packet::Gauge` |
| Timer | `name:value\|ms` | `latency:250\|ms` | `Packet::Timer` |
| Histogram | `name:value\|h` | `size:99.9\|h` | `Packet::Histogram` |
| Set | `name:value\|s` | `users:user123\|s` | `Packet::Set` |

### Sample Rate Support

```rust
Packet::Counter {
    name: "requests".to_string(),
    value: 10,
    sample_rate: Some(0.1),  // @0.1
}
```

**Note**: Tags (`#tag1,tag2`) are NOT currently supported - this is intentional to keep the parser simple.

## Code Quality Standards

### Linting Configuration

```rust
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::module_name_repetitions)]  // CapturedPackets is fine
#![allow(clippy::missing_errors_doc)]       // Error types are self-documenting
#![allow(clippy::missing_panics_doc)]       // Panic conditions are in tests
```

### Must-Use Annotations

All query methods are marked `#[must_use]`:

```rust
#[must_use]
pub fn counter(&self, name: &str) -> Option<i64>
```

**Rationale**: Calling these methods without using the result is likely a bug.

### Error Handling Philosophy

```rust
pub enum ParseError {
    InvalidFormat(String),
    InvalidValue(String),
    UnknownMetricType(String),
}
```

**Pattern**: Specific error types with context. Implement `Display` and `Error` trait.

**Usage**: Parse errors are logged but don't fail tests - gracefully skip malformed packets.

## Testing Strategy

### Test Organization

```rust
#[cfg(test)]
mod tests {
    // Basic functionality (5 tests)
    #[test] fn test_get_addr() { ... }

    // Packet parsing (8 tests)
    #[test] fn test_parse_counter() { ... }
    #[test] fn test_parse_invalid_format() { ... }

    // Lookup methods (3 tests)
    #[test] fn test_captured_packets_counter_lookup() { ... }

    // Fluent assertions (6 tests)
    #[test] fn test_captured_packets_assert_counter() { ... }
    #[test] fn test_captured_packets_fluent_assertions() { ... }

    // New features (20+ tests)
    #[test] fn test_histogram_lookup() { ... }
    #[test] fn test_all_counters() { ... }
    #[test] fn test_iterator_support() { ... }
}
```

### Test Coverage Requirements

Every public method must have:
1. **Happy path test** - Normal usage succeeds
2. **Error case test** - Invalid input handled gracefully
3. **Integration test** - Works with real `statsd` crate

Example:
```rust
#[test]
fn test_histogram_lookup() {
    let packets = CapturedPackets::from_raw(vec!["app.h:99.9|h".to_string()]);
    assert_eq!(packets.histogram("app.h"), Some(99.9));
    assert_eq!(packets.histogram("nonexistent"), None);  // Error case
}
```

## Common Development Tasks

### Adding a New Metric Type

1. **Add enum variant**:
```rust
pub enum Packet {
    // ...
    Distribution { name: String, value: f64 },
}
```

2. **Update parser**:
```rust
match metric_type {
    // ...
    "d" => Ok(Packet::Distribution { name, value: parse_f64(value_str)? }),
}
```

3. **Add accessor**:
```rust
pub fn as_distribution(&self) -> Option<f64> {
    match self { Packet::Distribution { value, .. } => Some(*value), _ => None }
}
```

4. **Add lookup helper**:
```rust
#[must_use]
pub fn distribution(&self, name: &str) -> Option<f64> {
    self.packets.iter().find(|p| p.name() == name).and_then(|p| p.as_distribution())
}
```

5. **Add assertion**:
```rust
pub fn assert_distribution(self, name: &str, expected: f64) -> Self {
    // Similar to assert_gauge
}
```

6. **Add tests** (minimum 5):
```rust
#[test] fn test_parse_distribution() { ... }
#[test] fn test_distribution_lookup() { ... }
#[test] fn test_assert_distribution() { ... }
#[test] fn test_assert_distribution_wrong_value() { ... }
#[test] fn test_assert_distribution_not_found() { ... }
```

### Adding a Collection Helper

1. **Decide return type**: `Vec<(&str, T)>` for name-value pairs
2. **Implement**:
```rust
#[must_use]
pub fn all_timers(&self) -> Vec<(&str, f64)> {
    self.packets.iter().filter_map(|p| match p {
        Packet::Timer { name, value } => Some((name.as_str(), *value)),
        _ => None,
    }).collect()
}
```
3. **Add test**:
```rust
#[test]
fn test_all_timers() {
    let raw = vec!["a.t:1.0|ms".to_string(), "b.t:2.0|ms".to_string()];
    let packets = CapturedPackets::from_raw(raw);
    let timers = packets.all_timers();
    assert_eq!(timers.len(), 2);
    assert!(timers.contains(&("a.t", 1.0)));
}
```

### Updating Dependencies

**Before upgrading**:
1. Check MSRV compatibility: `cargo +1.62.0 check`
2. Run full test suite: `cargo test`
3. Check clippy: `cargo clippy -- -D warnings`

**Example** (itertools upgrade):
```toml
# Check: Does itertools X.Y support Rust 1.62.0?
# Check transitive deps (especially `either` crate)
[dependencies]
itertools = "0.10"  # Safe for Rust 1.62.0
# itertools = "0.11"  # Requires Rust 1.63.0 - BREAKS MSRV!
```

## Performance Considerations

### UDP Buffer Size

```rust
let mut buf = [0; 1500];  // MTU size
```

**Rationale**: Standard Ethernet MTU. StatsD packets are typically <100 bytes.

**Don't change** unless you have evidence of larger packets.

### String Allocation

```rust
// ❌ Bad: Allocates on every lookup
pub fn counter(&self, name: &str) -> Option<i64> {
    self.packets.iter().find(|p| p.name().to_string() == name.to_string())
}

// ✅ Good: No allocation
pub fn counter(&self, name: &str) -> Option<i64> {
    self.packets.iter().find(|p| p.name() == name)
}
```

**Pattern**: Use `&str` comparisons, avoid `to_string()` in hot paths.

### Packet Collection

The intelligent timing loop is optimized for:
- **Low latency**: 10ms poll interval catches packets quickly
- **Low overhead**: Sleeps between polls, doesn't spin
- **Early exit**: Stops as soon as quiet period detected

**Don't**: Reduce poll interval below 10ms (wastes CPU for minimal gain).

## Backward Compatibility Checklist

Before releasing:

- [ ] All existing tests pass unchanged
- [ ] No public API removed or changed
- [ ] Old examples still compile and run
- [ ] Documentation shows both old and new ways
- [ ] MSRV still 1.62.0 (run `cargo +1.62.0 check`)

Example of maintaining compatibility:

```rust
// ✅ Old API still works
let response = mock.capture(|| client.incr("counter"));
assert_eq!(response, "app.counter:1|c");

// ✅ New API is opt-in
let packets = mock.capture_parsed(|| client.incr("counter"));
assert_eq!(packets.counter("app.counter"), Some(1));
```

## Common Pitfalls

### 1. Race Conditions

**Problem**: Closure ends before UDP packets arrive.

**Solution**: Intelligent waiting with quiet period detection (already implemented).

### 2. Reusing Server

**Problem**: User tries to call `capture()` multiple times on same server.

**Solution**: API consumes `self`, preventing reuse (by design).

```rust
let mock = start();
mock.capture(|| { ... });  // ✅ Works
mock.capture(|| { ... });  // ❌ Compile error - mock was moved
```

### 3. Float Comparison

**Problem**: `assert_eq!(actual, expected)` fails for floats due to precision.

**Solution**: Use epsilon comparison:

```rust
assert!((actual - expected).abs() < f64::EPSILON);
```

### 4. Test Flakiness

**Problem**: Tests pass locally but fail in CI.

**Solution**: Intelligent timing handles this. If flakiness appears:
1. Check if QUIET_PERIOD_MS needs adjustment (currently 50ms)
2. Verify MAX_WAIT_SECS is sufficient (currently 2s)
3. Never reduce timeouts - increase if needed

## Release Process

1. **Version bump**: Update `Cargo.toml` version following semver
2. **Run full tests**: `cargo test`
3. **Check all Rust versions**: CI tests stable/beta/nightly/1.62.0
4. **Update README**: Add to "What's New" section
5. **Commit with conventional commit message**: `feat:`, `fix:`, `chore:`
6. **Tag release**: `git tag v0.2.0`
7. **Push**: `git push origin main --tags`
8. **Publish**: `cargo publish` (automated via CI on tags)

## Future Enhancement Ideas

Ideas for future versions (maintain backward compatibility):

### 1. Configurable Timing
```rust
StatsDServer::new()
    .with_quiet_period(Duration::from_millis(100))
    .with_max_wait(Duration::from_secs(5))
```

### 2. Packet Filtering
```rust
packets.filter(|p| p.name().starts_with("myapp."))
```

### 3. Aggregate Statistics
```rust
packets.sum_counters()  // Sum all counter values
packets.avg_timers()    // Average all timer values
```

### 4. Snapshot Testing
```rust
packets.snapshot()  // For regression testing
```

### 5. Async Support
```rust
async fn capture_async<F: Future>(self, func: F) -> CapturedPackets
```

**Note**: All enhancements must preserve backward compatibility.

## Questions?

This is a living document. If you encounter scenarios not covered here:

1. Check existing code for patterns
2. Prioritize simplicity over cleverness
3. Add tests for any new behavior
4. Update this document with your findings

---

**Last Updated**: 2025-01-16
**MSRV**: Rust 1.62.0
**Version**: 0.2.0
