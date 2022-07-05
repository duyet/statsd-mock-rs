## Mock for StasD

Mocking for [stasd](https://docs.rs/statsd) crate. 

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
statsd-mock = "0.1"
```

### Example

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

## License

[MIT](LICENSE.txt).


