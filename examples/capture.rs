use statsd::client::Client;

fn main() {
    let mock = statsd_mock::start();

    let client = Client::new(&mock.addr(), "myapp").unwrap();
    let response = mock.capture(|| client.incr("some.counter"));

    assert_eq!(response, "myapp.some.counter:1|c");
}
