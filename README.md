## Mock for StasD

[![Rust](https://github.com/duyet/stasd-mock-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/duyet/stasd-mock-rs/actions/workflows/rust.yml)
[![codecov](https://codecov.io/gh/duyet/stasd-mock-rs/branch/master/graph/badge.svg?token=UCI27SSTAR)](https://codecov.io/gh/duyet/stasd-mock-rs)

Mocking for [stasd](https://docs.rs/statsd) crate. 

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


