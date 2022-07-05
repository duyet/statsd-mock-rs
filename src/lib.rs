#![doc = include_str!("../README.md")]

use std::net::{SocketAddr, UdpSocket};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::channel,
    Arc,
};
use std::thread;
use std::time::Duration;

// Mock StatsD Server
pub struct StatsDServer {
    local_addr: SocketAddr,
    sock: UdpSocket,
}

impl Default for StatsDServer {
    fn default() -> Self {
        Self::new()
    }
}

impl StatsDServer {
    pub fn new() -> Self {
        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let sock = UdpSocket::bind(addr).unwrap();

        sock.set_read_timeout(Some(Duration::from_millis(100)))
            .unwrap();
        let local_addr = sock.local_addr().unwrap();

        Self { local_addr, sock }
    }

    /// Return the mock server address: `127.0.0.1:<random port>`
    pub fn addr(&self) -> String {
        self.local_addr.clone().to_string()
    }

    /// Run the given test function while receiving several packets.
    /// Return a vector of the packets.
    ///
    /// ```
    /// use statsd::Client;
    ///
    /// // Start the mock server
    /// let mock = statsd_mock::start();
    ///
    /// let client = Client::new(&mock.addr(), "duyet").unwrap();
    /// let response = mock.run_while_receiving_all(|| {
    ///     client.incr("some.counter");
    ///     client.count("some.counter", 123.0);
    /// });
    /// assert_eq!(
    ///     response,
    ///     vec!["duyet.some.counter:1|c", "duyet.some.counter:123|c"]
    /// );
    /// ```
    pub fn run_while_receiving_all<F>(self, func: F) -> Vec<String>
    where
        F: Fn(),
    {
        let (serv_tx, serv_rx) = channel();
        let func_ran = Arc::new(AtomicBool::new(false));
        let bg_func_ran = Arc::clone(&func_ran);

        let bg = thread::spawn(move || loop {
            let mut buf = [0; 1500];
            if let Ok((len, _)) = self.sock.recv_from(&mut buf) {
                let bytes = Vec::from(&buf[0..len]);
                serv_tx.send(bytes).unwrap();
            }
            // go through the loop least once (do...while)
            if bg_func_ran.load(Ordering::SeqCst) {
                break;
            }
        });

        func();

        std::thread::sleep(Duration::from_millis(200));
        func_ran.store(true, Ordering::SeqCst);
        bg.join().expect("background thread should join");

        serv_rx
            .into_iter()
            .map(|bytes| String::from_utf8(bytes).unwrap())
            .collect()
    }

    /// Run the given test function while receiving several packets.
    /// Return a vector of the packets.
    pub fn capture_all<F>(self, func: F) -> Vec<String>
    where
        F: Fn(),
    {
        self.run_while_receiving_all(func)
    }

    /// Run the given test function while receiving several packets.
    /// Return the concatenation of the packets.
    ///
    /// ```
    /// use statsd::Client;
    ///
    /// // Start the mock server
    /// let mock = statsd_mock::start();
    ///
    /// let client = Client::new(&mock.addr(), "duyet").unwrap();
    /// let response = mock.run_while_receiving(|| {
    ///     client.count("some.counter", 123.0);
    /// });
    /// assert_eq!(response, "duyet.some.counter:123|c");
    /// ```
    pub fn run_while_receiving<F>(self, func: F) -> String
    where
        F: Fn(),
    {
        itertools::Itertools::intersperse(
            self.run_while_receiving_all(func).into_iter(),
            String::from("\n"),
        )
        .fold(String::new(), |acc, b| acc + &b)
    }

    /// Run the given test function while receiving several packets.
    /// Return the concatenation of the packets.
    pub fn capture<F>(self, func: F) -> String
    where
        F: Fn(),
    {
        self.run_while_receiving(func)
    }
}

pub fn start() -> StatsDServer {
    StatsDServer::default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use statsd::client::Client;

    #[test]
    fn test_get_addr() {
        let mock = start();

        assert_eq!(mock.addr().contains("127.0.0.1:"), true);
    }

    #[test]
    fn test_capture_incr() {
        let mock = start();

        let client = Client::new(&mock.addr(), "duyet").unwrap();
        let response = mock.capture(|| client.incr("some.counter"));

        assert_eq!(response, "duyet.some.counter:1|c");
    }

    #[test]
    fn test_capture_decr() {
        let mock = start();

        let client = Client::new(&mock.addr(), "duyet").unwrap();
        let response = mock.capture(|| client.decr("some.counter"));

        assert_eq!(response, "duyet.some.counter:-1|c");
    }

    #[test]
    fn test_capture_count() {
        let mock = start();

        let client = Client::new(&mock.addr(), "duyet").unwrap();
        let response = mock.capture(|| {
            client.count("some.counter", 123.0);
        });

        assert_eq!(response, "duyet.some.counter:123|c");
    }

    #[test]
    fn test_capture_all() {
        let mock = start();

        let client = Client::new(&mock.addr(), "duyet").unwrap();
        let response = mock.capture_all(|| {
            client.incr("some.counter");
            client.incr("some.counter2");
            client.count("some.counter3", 123.0);
        });

        assert_eq!(
            response,
            vec![
                "duyet.some.counter:1|c",
                "duyet.some.counter2:1|c",
                "duyet.some.counter3:123|c"
            ]
        );
    }
}
