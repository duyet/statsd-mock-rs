# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added ✨

- **Structured Packet Parsing** - New `Packet` enum for type-safe representation of StatsD metrics
  - Parse counters, gauges, timers, histograms, and sets
  - Extract metric names and values with compile-time safety
  - Support for sample rates on counters

- **`capture_parsed()` Method** - New recommended API that returns `CapturedPackets`
  - Type-safe value lookups: `packets.counter("name")`, `packets.gauge("name")`, etc.
  - Fluent chainable assertions: `.assert_counter().assert_gauge().assert_timer()`
  - Access to both parsed packets and raw strings for debugging

- **Intelligent Packet Collection** - Replaced arbitrary 200ms sleep with adaptive timing
  - Waits until "quiet period" is detected (no packets for 50ms)
  - Adapts to actual network timing instead of fixed delays
  - Faster on fast machines, more reliable on slow CI systems
  - 2-second safety timeout prevents hanging

- **Comprehensive Test Coverage** - 20+ new tests for packet parsing and assertions
  - All StatsD metric types covered
  - Error handling validated
  - Integration tests for full workflow

- **Advanced Example** - New `examples/advanced.rs` demonstrating the parsed API

### Changed 🔄

- **Timing Logic** - Internal packet collection now uses intelligent waiting instead of `sleep(200ms)`
  - Uses `recv_timeout()` with quiet period detection
  - Significantly reduces test flakiness
  - Typically faster than the old fixed 200ms delay

### Fixed 🐛

- **Race Condition** - The 200ms hardcoded sleep could miss packets on slow systems or waste time on fast ones
  - Now adaptively waits until all packets are received
  - More reliable in CI environments

## [0.1.1] - Previous Release

### Changed
- Dependency updates via Renovate

## [0.1.0] - Initial Release

### Added
- Basic StatsD mock server for testing
- `capture()` method for capturing packets as concatenated string
- `capture_all()` method for capturing packets as vector of strings
- UDP server with automatic port allocation
- Thread-based packet collection

---

## Migration Guide

### Upgrading to New API

The new `capture_parsed()` API is **100% backward compatible**. Existing code continues to work unchanged.

#### Before (still works!):
```rust
let response = mock.capture(|| client.incr("counter"));
assert_eq!(response, "myapp.counter:1|c");
```

#### After (recommended for new code):
```rust
let packets = mock.capture_parsed(|| client.incr("counter"));
assert_eq!(packets.counter("myapp.counter"), Some(1));

// Or use fluent assertions:
packets.assert_counter("myapp.counter", 1);
```

### Benefits of Migrating

1. **Type Safety** - Compiler helps catch typos in metric names
2. **Cleaner Tests** - No string parsing in your test assertions
3. **Better Errors** - Descriptive panic messages when assertions fail
4. **More Flexible** - Easy to check specific metrics in multi-metric tests
5. **Future Proof** - Ready for additional StatsD protocol features

### No Breaking Changes

- All existing methods remain unchanged
- Performance is equal or better (thanks to intelligent timing)
- Zero API surface removed
- Drop-in replacement - just upgrade the version!
