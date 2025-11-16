/// Example demonstrating error handling and packet parsing
///
/// This shows how to use the Packet::parse() method directly and handle
/// parsing errors gracefully.

use statsd_mock::{Packet, ParseError};

fn main() {
    println!("=== Packet Parsing Error Handling ===\n");

    // Valid packets
    let valid_packets = vec![
        "myapp.counter:1|c",
        "myapp.gauge:42.5|g",
        "myapp.timer:100|ms",
        "myapp.histogram:99.9|h",
        "myapp.users:user123|s",
        "myapp.sampled:10|c|@0.5",
    ];

    println!("Parsing valid packets:");
    for packet_str in valid_packets {
        match Packet::parse(packet_str) {
            Ok(packet) => {
                println!("  ✓ Parsed: {} -> {:?}", packet_str, packet);
                println!("    Name: {}", packet.name());
                println!("    Display: {}\n", packet);
            }
            Err(e) => {
                println!("  ✗ Error: {}", e);
            }
        }
    }

    // Invalid packets
    let invalid_packets = vec![
        ("invalid", "missing pipe separator"),
        ("name|c", "missing colon in name:value"),
        ("name:value", "missing metric type"),
        ("name:notanumber|c", "invalid counter value"),
        ("name:value|unknown", "unknown metric type"),
    ];

    println!("\n=== Parsing invalid packets ===");
    for (packet_str, description) in invalid_packets {
        println!("\nTesting: {} ({})", packet_str, description);
        match Packet::parse(packet_str) {
            Ok(packet) => {
                println!("  Unexpected success: {:?}", packet);
            }
            Err(ParseError::InvalidFormat(msg)) => {
                println!("  ✓ Caught InvalidFormat: {}", msg);
            }
            Err(ParseError::InvalidValue(msg)) => {
                println!("  ✓ Caught InvalidValue: {}", msg);
            }
            Err(ParseError::UnknownMetricType(msg)) => {
                println!("  ✓ Caught UnknownMetricType: {}", msg);
            }
        }
    }

    println!("\n=== Working with malformed data ===");

    // Simulate capturing potentially malformed packets
    let mixed_packets = vec![
        "good.counter:1|c",
        "bad-packet-missing-parts",
        "good.gauge:42.0|g",
        "another.bad:value|x",
        "good.timer:100|ms",
    ];

    let parsed: Vec<_> = mixed_packets
        .iter()
        .filter_map(|s| {
            match Packet::parse(s) {
                Ok(p) => Some(p),
                Err(e) => {
                    eprintln!("Warning: Failed to parse '{}': {}", s, e);
                    None
                }
            }
        })
        .collect();

    println!("\nSuccessfully parsed {} out of {} packets", parsed.len(), mixed_packets.len());
    for packet in parsed {
        println!("  - {}: {}", packet.name(), packet);
    }

    println!("\n✓ Error handling example completed!");
}
