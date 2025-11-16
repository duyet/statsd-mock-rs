#![doc = include_str!("../README.md")]

use std::net::{SocketAddr, UdpSocket};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{self, channel},
    Arc,
};
use std::thread;
use std::time::{Duration, Instant};

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

        // Intelligent packet collection: wait until we see a quiet period
        // This adapts to actual network timing instead of arbitrary sleeps
        let mut packets = Vec::new();
        let start = Instant::now();
        let max_wait = Duration::from_secs(2);
        let quiet_period = Duration::from_millis(50);
        let mut last_packet_time = Instant::now();

        loop {
            match serv_rx.recv_timeout(Duration::from_millis(10)) {
                Ok(bytes) => {
                    last_packet_time = Instant::now();
                    packets.push(bytes);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // No packet in 10ms - check if we're done
                    if last_packet_time.elapsed() >= quiet_period {
                        break; // Quiet period detected - we're done
                    }
                    if start.elapsed() >= max_wait {
                        break; // Safety timeout
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    break; // Channel closed
                }
            }
        }

        // Signal background thread to stop
        func_ran.store(true, Ordering::SeqCst);
        bg.join().expect("background thread should join");

        packets
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

    /// Run the given test function while receiving several packets.
    /// Return a CapturedPackets with parsed packet data for easier assertions.
    ///
    /// This is the recommended API for new code as it provides type-safe
    /// access to metric values and chainable assertions.
    ///
    /// # Examples
    ///
    /// ```
    /// use statsd::Client;
    ///
    /// let mock = statsd_mock::start();
    /// let client = Client::new(&mock.addr(), "myapp").unwrap();
    ///
    /// let packets = mock.capture_parsed(|| {
    ///     client.incr("requests");
    ///     client.gauge("memory", 42.0);
    ///     client.time("response_time", 100.0);
    /// });
    ///
    /// // Type-safe value access
    /// assert_eq!(packets.counter("myapp.requests"), Some(1));
    /// assert_eq!(packets.gauge("myapp.memory"), Some(42.0));
    ///
    /// // Or use fluent assertions
    /// packets
    ///     .assert_counter("myapp.requests", 1)
    ///     .assert_gauge("myapp.memory", 42.0)
    ///     .assert_timer("myapp.response_time", 100.0);
    /// ```
    pub fn capture_parsed<F>(self, func: F) -> CapturedPackets
    where
        F: Fn(),
    {
        let raw = self.run_while_receiving_all(func);
        CapturedPackets::from_raw(raw)
    }
}

pub fn start() -> StatsDServer {
    StatsDServer::default()
}

// ============================================================================
// Structured Packet Parsing
// ============================================================================

/// A parsed StatsD metric packet
#[derive(Debug, Clone, PartialEq)]
pub enum Packet {
    /// Counter: name:value|c[|@sample_rate]
    Counter {
        name: String,
        value: i64,
        sample_rate: Option<f64>,
    },
    /// Gauge: name:value|g
    Gauge { name: String, value: f64 },
    /// Timer: name:value|ms
    Timer { name: String, value: f64 },
    /// Histogram: name:value|h
    Histogram { name: String, value: f64 },
    /// Set: name:value|s
    Set { name: String, value: String },
}

/// Errors that can occur when parsing StatsD packets
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    InvalidFormat(String),
    InvalidValue(String),
    UnknownMetricType(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidFormat(s) => write!(f, "Invalid packet format: {}", s),
            ParseError::InvalidValue(s) => write!(f, "Invalid value: {}", s),
            ParseError::UnknownMetricType(s) => write!(f, "Unknown metric type: {}", s),
        }
    }
}

impl std::error::Error for ParseError {}

impl Packet {
    /// Parse a StatsD protocol string into a structured Packet
    ///
    /// # Examples
    ///
    /// ```
    /// use statsd_mock::Packet;
    ///
    /// let packet = Packet::parse("myapp.counter:1|c").unwrap();
    /// assert_eq!(packet.name(), "myapp.counter");
    /// ```
    pub fn parse(s: &str) -> Result<Self, ParseError> {
        // Format: name:value|type[|@sample_rate]
        let parts: Vec<&str> = s.split('|').collect();
        if parts.len() < 2 {
            return Err(ParseError::InvalidFormat(
                "Expected format: name:value|type".to_string(),
            ));
        }

        // Parse name:value
        let name_value: Vec<&str> = parts[0].split(':').collect();
        if name_value.len() != 2 {
            return Err(ParseError::InvalidFormat(
                "Expected name:value before |".to_string(),
            ));
        }

        let name = name_value[0].to_string();
        let value_str = name_value[1];
        let metric_type = parts[1];

        // Parse optional sample rate (for counters)
        let sample_rate = if parts.len() > 2 && parts[2].starts_with('@') {
            parts[2]
                .trim_start_matches('@')
                .parse::<f64>()
                .ok()
        } else {
            None
        };

        // Parse based on metric type
        match metric_type {
            "c" => {
                let value = value_str
                    .parse::<i64>()
                    .map_err(|_| ParseError::InvalidValue(value_str.to_string()))?;
                Ok(Packet::Counter {
                    name,
                    value,
                    sample_rate,
                })
            }
            "g" => {
                let value = value_str
                    .parse::<f64>()
                    .map_err(|_| ParseError::InvalidValue(value_str.to_string()))?;
                Ok(Packet::Gauge { name, value })
            }
            "ms" => {
                let value = value_str
                    .parse::<f64>()
                    .map_err(|_| ParseError::InvalidValue(value_str.to_string()))?;
                Ok(Packet::Timer { name, value })
            }
            "h" => {
                let value = value_str
                    .parse::<f64>()
                    .map_err(|_| ParseError::InvalidValue(value_str.to_string()))?;
                Ok(Packet::Histogram { name, value })
            }
            "s" => Ok(Packet::Set {
                name,
                value: value_str.to_string(),
            }),
            _ => Err(ParseError::UnknownMetricType(metric_type.to_string())),
        }
    }

    /// Get the metric name
    pub fn name(&self) -> &str {
        match self {
            Packet::Counter { name, .. } => name,
            Packet::Gauge { name, .. } => name,
            Packet::Timer { name, .. } => name,
            Packet::Histogram { name, .. } => name,
            Packet::Set { name, .. } => name,
        }
    }

    /// Get the value as a counter (if applicable)
    pub fn as_counter(&self) -> Option<i64> {
        match self {
            Packet::Counter { value, .. } => Some(*value),
            _ => None,
        }
    }

    /// Get the value as a gauge (if applicable)
    pub fn as_gauge(&self) -> Option<f64> {
        match self {
            Packet::Gauge { value, .. } => Some(*value),
            _ => None,
        }
    }

    /// Get the value as a timer (if applicable)
    pub fn as_timer(&self) -> Option<f64> {
        match self {
            Packet::Timer { value, .. } => Some(*value),
            _ => None,
        }
    }
}

/// A collection of captured packets with helper methods for assertions
#[derive(Debug, Clone)]
pub struct CapturedPackets {
    packets: Vec<Packet>,
    raw: Vec<String>,
}

impl CapturedPackets {
    /// Create a new CapturedPackets from raw strings
    pub fn from_raw(raw: Vec<String>) -> Self {
        let packets = raw
            .iter()
            .filter_map(|s| Packet::parse(s).ok())
            .collect();
        Self { packets, raw }
    }

    /// Get all packets
    pub fn packets(&self) -> &[Packet] {
        &self.packets
    }

    /// Get raw strings (for backward compatibility)
    pub fn raw(&self) -> &[String] {
        &self.raw
    }

    /// Number of captured packets
    pub fn len(&self) -> usize {
        self.packets.len()
    }

    /// Check if no packets were captured
    pub fn is_empty(&self) -> bool {
        self.packets.is_empty()
    }

    /// Find a counter by name and return its value
    pub fn counter(&self, name: &str) -> Option<i64> {
        self.packets
            .iter()
            .find(|p| p.name() == name)
            .and_then(|p| p.as_counter())
    }

    /// Find a gauge by name and return its value
    pub fn gauge(&self, name: &str) -> Option<f64> {
        self.packets
            .iter()
            .find(|p| p.name() == name)
            .and_then(|p| p.as_gauge())
    }

    /// Find a timer by name and return its value
    pub fn timer(&self, name: &str) -> Option<f64> {
        self.packets
            .iter()
            .find(|p| p.name() == name)
            .and_then(|p| p.as_timer())
    }

    /// Chainable assertion for counter values
    ///
    /// # Panics
    ///
    /// Panics if the counter is not found or has a different value
    pub fn assert_counter(self, name: &str, expected: i64) -> Self {
        match self.counter(name) {
            Some(actual) => {
                assert_eq!(
                    actual, expected,
                    "Counter '{}' has value {} but expected {}",
                    name, actual, expected
                );
            }
            None => panic!("Counter '{}' not found in captured packets", name),
        }
        self
    }

    /// Chainable assertion for gauge values
    ///
    /// # Panics
    ///
    /// Panics if the gauge is not found or has a different value
    pub fn assert_gauge(self, name: &str, expected: f64) -> Self {
        match self.gauge(name) {
            Some(actual) => {
                assert!(
                    (actual - expected).abs() < f64::EPSILON,
                    "Gauge '{}' has value {} but expected {}",
                    name,
                    actual,
                    expected
                );
            }
            None => panic!("Gauge '{}' not found in captured packets", name),
        }
        self
    }

    /// Chainable assertion for timer values
    ///
    /// # Panics
    ///
    /// Panics if the timer is not found or has a different value
    pub fn assert_timer(self, name: &str, expected: f64) -> Self {
        match self.timer(name) {
            Some(actual) => {
                assert!(
                    (actual - expected).abs() < f64::EPSILON,
                    "Timer '{}' has value {} but expected {}",
                    name,
                    actual,
                    expected
                );
            }
            None => panic!("Timer '{}' not found in captured packets", name),
        }
        self
    }
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

    // ========================================================================
    // Tests for Packet Parsing
    // ========================================================================

    #[test]
    fn test_parse_counter() {
        let packet = Packet::parse("myapp.counter:1|c").unwrap();
        assert_eq!(packet.name(), "myapp.counter");
        assert_eq!(packet.as_counter(), Some(1));

        match packet {
            Packet::Counter {
                name,
                value,
                sample_rate,
            } => {
                assert_eq!(name, "myapp.counter");
                assert_eq!(value, 1);
                assert_eq!(sample_rate, None);
            }
            _ => panic!("Expected counter packet"),
        }
    }

    #[test]
    fn test_parse_counter_with_sample_rate() {
        let packet = Packet::parse("myapp.counter:5|c|@0.5").unwrap();
        match packet {
            Packet::Counter {
                name,
                value,
                sample_rate,
            } => {
                assert_eq!(name, "myapp.counter");
                assert_eq!(value, 5);
                assert_eq!(sample_rate, Some(0.5));
            }
            _ => panic!("Expected counter packet"),
        }
    }

    #[test]
    fn test_parse_negative_counter() {
        let packet = Packet::parse("myapp.counter:-1|c").unwrap();
        assert_eq!(packet.as_counter(), Some(-1));
    }

    #[test]
    fn test_parse_gauge() {
        let packet = Packet::parse("myapp.memory:42.5|g").unwrap();
        assert_eq!(packet.name(), "myapp.memory");
        assert_eq!(packet.as_gauge(), Some(42.5));
    }

    #[test]
    fn test_parse_timer() {
        let packet = Packet::parse("myapp.response_time:123.456|ms").unwrap();
        assert_eq!(packet.name(), "myapp.response_time");
        assert_eq!(packet.as_timer(), Some(123.456));
    }

    #[test]
    fn test_parse_histogram() {
        let packet = Packet::parse("myapp.data:99.9|h").unwrap();
        assert_eq!(packet.name(), "myapp.data");
        match packet {
            Packet::Histogram { value, .. } => assert_eq!(value, 99.9),
            _ => panic!("Expected histogram packet"),
        }
    }

    #[test]
    fn test_parse_set() {
        let packet = Packet::parse("myapp.users:user123|s").unwrap();
        assert_eq!(packet.name(), "myapp.users");
        match packet {
            Packet::Set { value, .. } => assert_eq!(value, "user123"),
            _ => panic!("Expected set packet"),
        }
    }

    #[test]
    fn test_parse_invalid_format() {
        let result = Packet::parse("invalid");
        assert!(result.is_err());
        match result {
            Err(ParseError::InvalidFormat(_)) => {}
            _ => panic!("Expected InvalidFormat error"),
        }
    }

    #[test]
    fn test_parse_unknown_metric_type() {
        let result = Packet::parse("myapp.metric:123|unknown");
        assert!(result.is_err());
        match result {
            Err(ParseError::UnknownMetricType(t)) => assert_eq!(t, "unknown"),
            _ => panic!("Expected UnknownMetricType error"),
        }
    }

    #[test]
    fn test_parse_invalid_counter_value() {
        let result = Packet::parse("myapp.counter:notanumber|c");
        assert!(result.is_err());
        match result {
            Err(ParseError::InvalidValue(_)) => {}
            _ => panic!("Expected InvalidValue error"),
        }
    }

    #[test]
    fn test_captured_packets_counter_lookup() {
        let raw = vec![
            "myapp.counter1:1|c".to_string(),
            "myapp.counter2:42|c".to_string(),
            "myapp.gauge:3.14|g".to_string(),
        ];
        let packets = CapturedPackets::from_raw(raw);

        assert_eq!(packets.len(), 3);
        assert_eq!(packets.counter("myapp.counter1"), Some(1));
        assert_eq!(packets.counter("myapp.counter2"), Some(42));
        assert_eq!(packets.counter("nonexistent"), None);
    }

    #[test]
    fn test_captured_packets_gauge_lookup() {
        let raw = vec![
            "myapp.memory:42.5|g".to_string(),
            "myapp.cpu:99.9|g".to_string(),
        ];
        let packets = CapturedPackets::from_raw(raw);

        assert_eq!(packets.gauge("myapp.memory"), Some(42.5));
        assert_eq!(packets.gauge("myapp.cpu"), Some(99.9));
        assert_eq!(packets.gauge("nonexistent"), None);
    }

    #[test]
    fn test_captured_packets_timer_lookup() {
        let raw = vec!["myapp.response_time:123.456|ms".to_string()];
        let packets = CapturedPackets::from_raw(raw);

        assert_eq!(packets.timer("myapp.response_time"), Some(123.456));
        assert_eq!(packets.timer("nonexistent"), None);
    }

    #[test]
    fn test_captured_packets_raw_access() {
        let raw = vec![
            "myapp.counter:1|c".to_string(),
            "myapp.gauge:42.5|g".to_string(),
        ];
        let packets = CapturedPackets::from_raw(raw.clone());

        assert_eq!(packets.raw(), &raw[..]);
    }

    #[test]
    fn test_captured_packets_assert_counter() {
        let raw = vec!["myapp.counter:42|c".to_string()];
        let packets = CapturedPackets::from_raw(raw);

        // Should not panic
        packets.assert_counter("myapp.counter", 42);
    }

    #[test]
    #[should_panic(expected = "Counter 'myapp.counter' has value 42 but expected 100")]
    fn test_captured_packets_assert_counter_wrong_value() {
        let raw = vec!["myapp.counter:42|c".to_string()];
        let packets = CapturedPackets::from_raw(raw);

        packets.assert_counter("myapp.counter", 100);
    }

    #[test]
    #[should_panic(expected = "Counter 'nonexistent' not found")]
    fn test_captured_packets_assert_counter_not_found() {
        let raw = vec!["myapp.counter:42|c".to_string()];
        let packets = CapturedPackets::from_raw(raw);

        packets.assert_counter("nonexistent", 42);
    }

    #[test]
    fn test_captured_packets_fluent_assertions() {
        let raw = vec![
            "myapp.counter:1|c".to_string(),
            "myapp.gauge:42.0|g".to_string(),
            "myapp.timer:100.0|ms".to_string(),
        ];
        let packets = CapturedPackets::from_raw(raw);

        // Should not panic - chainable assertions
        packets
            .assert_counter("myapp.counter", 1)
            .assert_gauge("myapp.gauge", 42.0)
            .assert_timer("myapp.timer", 100.0);
    }

    #[test]
    fn test_capture_parsed_integration() {
        let mock = start();
        let client = Client::new(&mock.addr(), "testapp").unwrap();

        let packets = mock.capture_parsed(|| {
            client.incr("requests");
            client.count("items", 5.0);
            client.gauge("memory", 1024.0);
        });

        assert_eq!(packets.len(), 3);
        assert_eq!(packets.counter("testapp.requests"), Some(1));
        assert_eq!(packets.counter("testapp.items"), Some(5));
        assert_eq!(packets.gauge("testapp.memory"), Some(1024.0));
    }
}
