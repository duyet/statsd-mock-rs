## Mock StatsD for Rust

Mock for [statsd](https://docs.rs/statsd) crate. 

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]
[![Build Status][actions-badge]][actions-url]
[![Codecov][codecov-badge]][codecov-url]

[crates-badge]: https://img.shields.io/crates/v/statsd-mock.svg
[crates-url]: https://crates.io/crates/statsd-mock
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/duyet/statsd-mock-rs/blob/master/LICENSE
[actions-badge]: https://github.com/duyet/statsd-mock-rs/actions/workflows/rust.yml/badge.svg
[actions-url]: https://github.com/duyet/statsd-mock-rs/actions?query=branch%3Amaster
[codecov-badge]: https://codecov.io/gh/duyet/stasd-mock-rs/branch/master/graph/badge.svg?token=UCI27SSTAR
[codecov-url]: https://codecov.io/gh/duyet/stasd-mock-rs

[Website](https://github.com/duyet/statsd-mock-rs) |
[API Docs](https://docs.rs/statsd-mock)

### Usage

Add the `statsd-mock` package as a dev dependency in your `Cargo.toml` file

```toml
[dev-dependencies]
statsd-mock = "0.2"
```

### Example

#### Simple String Matching (Original API)

```rust
use statsd::client::Client;

fn main() {
  // Start the mock server
  let mock = statsd_mock::start();

  // mock.addr() to get mock server address
  let client = Client::new(&mock.addr(), "myapp").unwrap();

  // Capturing
  let response = mock.capture(|| client.incr("some.counter"));

  assert_eq!(response, "myapp.some.counter:1|c");
}
```

#### Structured Assertions (New Recommended API) ✨

The new `capture_parsed()` API provides type-safe access to metric values and fluent assertions:

```rust
use statsd::client::Client;

fn main() {
  let mock = statsd_mock::start();
  let client = Client::new(&mock.addr(), "myapp").unwrap();

  // Capture with parsed packet data
  let packets = mock.capture_parsed(|| {
    client.incr("requests");
    client.gauge("memory_mb", 1024.0);
    client.time("response_ms", 250.0);
  });

  // Type-safe value lookups
  assert_eq!(packets.counter("myapp.requests"), Some(1));
  assert_eq!(packets.gauge("myapp.memory_mb"), Some(1024.0));
  assert_eq!(packets.timer("myapp.response_ms"), Some(250.0));

  // Or use fluent chainable assertions
  packets
    .assert_counter("myapp.requests", 1)
    .assert_gauge("myapp.memory_mb", 1024.0)
    .assert_timer("myapp.response_ms", 250.0);
}
```

### Features

- ✅ **Zero Configuration** - Just start and use
- ✅ **Type-Safe Assertions** - Parsed packet API with compile-time safety
- ✅ **Fluent Interface** - Chain assertions for clean test code
- ✅ **Intelligent Timing** - Adaptive packet collection (no more arbitrary sleeps!)
- ✅ **Backward Compatible** - All existing code continues to work
- ✅ **Comprehensive Protocol Support** - Counters, gauges, timers, histograms, sets
- ✅ **Iterator Support** - Iterate over captured packets naturally
- ✅ **Collection Helpers** - Get all counters, all gauges, filter by name
- ✅ **Error Handling** - Graceful parsing with detailed error types

### What's New in 0.2.0

- **Structured Packet Parsing** - Parse StatsD packets into type-safe `Packet` enum
- **Enhanced Assertions** - New `assert_histogram()`, `assert_set()`, `assert_len()`, `assert_exists()`
- **Collection Methods** - `all_counters()`, `all_gauges()`, `filter_by_name()`
- **Iterator Support** - Full `IntoIterator` implementation for `CapturedPackets`
- **Display Implementation** - Pretty-print packets with `Display` trait
- **Comprehensive Testing** - 40+ tests covering all functionality
- **Clippy Lints** - Strict linting for code quality
- **Better Docs** - Extensive documentation and examples

## License

[MIT](LICENSE.txt).


