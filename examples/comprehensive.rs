/// Comprehensive example showing all statsd-mock features
///
/// This example demonstrates every feature of the library including
/// all metric types, assertions, collection helpers, and iterators.

use statsd::client::Client;
use statsd_mock::{CapturedPackets, Packet};

fn main() {
    println!("=== Comprehensive statsd-mock Feature Showcase ===\n");

    // 1. Basic usage with string matching (original API)
    println!("1. Basic string matching:");
    let mock = statsd_mock::start();
    let client = Client::new(&mock.addr(), "app").unwrap();
    let response = mock.capture(|| client.incr("requests"));
    assert_eq!(response, "app.requests:1|c");
    println!("   ✓ Captured: {}", response);

    // 2. Structured parsing with type-safe access
    println!("\n2. Structured parsing:");
    let mock = statsd_mock::start();
    let client = Client::new(&mock.addr(), "app").unwrap();
    let packets = mock.capture_parsed(|| {
        client.incr("requests");
        client.count("items", 42.0);
        client.gauge("memory_mb", 1024.0);
        client.time("response_ms", 250.0);
    });

    println!("   ✓ Captured {} packets", packets.len());
    println!("   ✓ requests counter: {}", packets.counter("app.requests").unwrap());
    println!("   ✓ items counter: {}", packets.counter("app.items").unwrap());
    println!("   ✓ memory gauge: {}", packets.gauge("app.memory_mb").unwrap());
    println!("   ✓ response timer: {}", packets.timer("app.response_ms").unwrap());

    // 3. Fluent assertions
    println!("\n3. Fluent assertions:");
    packets
        .clone()
        .assert_len(4)
        .assert_exists("app.requests")
        .assert_counter("app.requests", 1)
        .assert_counter("app.items", 42)
        .assert_gauge("app.memory_mb", 1024.0)
        .assert_timer("app.response_ms", 250.0);
    println!("   ✓ All assertions passed");

    // 4. Collection helpers
    println!("\n4. Collection helpers:");
    let all_counters = packets.all_counters();
    println!("   All counters:");
    for (name, value) in all_counters {
        println!("     - {} = {}", name, value);
    }

    let all_gauges = packets.all_gauges();
    println!("   All gauges:");
    for (name, value) in all_gauges {
        println!("     - {} = {}", name, value);
    }

    // 5. Filtering
    println!("\n5. Filtering packets:");
    let app_packets = packets.filter_by_prefix("app.");
    println!("   ✓ Found {} packets with 'app.' prefix", app_packets.len());

    let request_packets = packets.filter_by_name("app.requests");
    println!("   ✓ Found {} packets named 'app.requests'", request_packets.len());

    // 6. Query methods
    println!("\n6. Query methods:");
    println!("   ✓ contains('app.requests'): {}", packets.contains("app.requests"));
    println!("   ✓ count('app.requests'): {}", packets.count("app.requests"));
    if let Some(first) = packets.first("app.requests") {
        println!("   ✓ first packet: {}", first);
    }

    // 7. Iterator support
    println!("\n7. Iterator support:");
    for (i, packet) in (&packets).into_iter().enumerate() {
        println!("   [{}] {} = {:?}", i + 1, packet.name(), packet);
    }

    // 8. Trait implementations
    println!("\n8. Trait implementations:");
    let empty = CapturedPackets::new();
    println!("   ✓ Default/new: {} packets", empty.len());

    let from_vec: CapturedPackets = vec!["test:1|c".to_string()].into();
    println!("   ✓ From<Vec<String>>: {} packets", from_vec.len());

    let from_packets: CapturedPackets = vec![Packet::Counter {
        name: "manual".to_string(),
        value: 999,
        sample_rate: None,
    }]
    .into();
    println!("   ✓ From<Vec<Packet>>: {} packets", from_packets.len());

    // 9. Server introspection
    println!("\n9. Server introspection:");
    let mock = statsd_mock::start();
    println!("   ✓ Address: {}", mock.addr());
    println!("   ✓ Port: {}", mock.port());

    // 10. Display trait for packets
    println!("\n10. Display trait:");
    let packet = Packet::Counter {
        name: "display.test".to_string(),
        value: 123,
        sample_rate: Some(0.5),
    };
    println!("   ✓ Packet display: {}", packet);

    println!("\n=== All features demonstrated successfully! ===");
}
