/// Advanced example demonstrating the new parsed packet API
///
/// This shows how to use the structured packet parsing and fluent assertions
/// for more ergonomic testing.

use statsd::client::Client;

fn main() {
    println!("=== Advanced statsd-mock Example ===\n");

    // Start the mock server
    let mock = statsd_mock::start();
    println!("Mock server started at: {}", mock.addr());

    // Create a StatsD client pointing to our mock
    let client = Client::new(&mock.addr(), "myapp").unwrap();

    // Capture packets with the new parsed API
    let packets = mock.capture_parsed(|| {
        println!("\nSending metrics...");

        // Send various metric types
        client.incr("requests");
        client.count("items_processed", 42.0);
        client.gauge("memory_mb", 1024.0);
        client.time("response_time_ms", 250.0);

        println!("✓ Sent 4 metrics");
    });

    println!("\n=== Captured {} packets ===\n", packets.len());

    // Type-safe value access
    println!("Type-safe lookups:");
    if let Some(requests) = packets.counter("myapp.requests") {
        println!("  ✓ requests counter: {}", requests);
    }
    if let Some(items) = packets.counter("myapp.items_processed") {
        println!("  ✓ items_processed counter: {}", items);
    }
    if let Some(memory) = packets.gauge("myapp.memory_mb") {
        println!("  ✓ memory_mb gauge: {}", memory);
    }
    if let Some(time) = packets.timer("myapp.response_time_ms") {
        println!("  ✓ response_time_ms timer: {}", time);
    }

    // Fluent assertions (these would panic if values don't match)
    println!("\n=== Running fluent assertions ===");
    packets
        .assert_counter("myapp.requests", 1)
        .assert_counter("myapp.items_processed", 42)
        .assert_gauge("myapp.memory_mb", 1024.0)
        .assert_timer("myapp.response_time_ms", 250.0);

    println!("✓ All assertions passed!");

    // Access raw packets if needed (backward compatibility)
    println!("\n=== Raw packets ===");
    for (i, raw) in packets.raw().iter().enumerate() {
        println!("  [{}] {}", i + 1, raw);
    }

    // Inspect parsed packet structure
    println!("\n=== Parsed packet details ===");
    for (i, packet) in packets.packets().iter().enumerate() {
        println!("  [{}] {:?}", i + 1, packet);
    }

    // Use collection helpers
    println!("\n=== Collection helpers ===");
    let all_counters = packets.all_counters();
    println!("All counters:");
    for (name, value) in all_counters {
        println!("  {} = {}", name, value);
    }

    let all_gauges = packets.all_gauges();
    println!("\nAll gauges:");
    for (name, value) in all_gauges {
        println!("  {} = {}", name, value);
    }

    // Iterator support
    println!("\n=== Iterator support ===");
    println!("Iterating over packets:");
    for (i, packet) in (&packets).into_iter().enumerate() {
        println!("  [{}] {} (name: {})", i + 1, packet, packet.name());
    }

    // Advanced assertions
    println!("\n=== Advanced assertions ===");
    let mock2 = statsd_mock::start();
    let client2 = Client::new(&mock2.addr(), "app").unwrap();

    mock2
        .capture_parsed(|| {
            client2.incr("requests");
            client2.incr("errors");
            client2.gauge("cpu", 75.5);
        })
        .assert_len(3) // Assert exactly 3 packets
        .assert_exists("app.requests") // Assert metric exists
        .assert_exists("app.errors")
        .assert_counter("app.requests", 1)
        .assert_counter("app.errors", 1)
        .assert_gauge("app.cpu", 75.5);

    println!("✓ All advanced assertions passed!");

    println!("\n✓ Example completed successfully!");
}
