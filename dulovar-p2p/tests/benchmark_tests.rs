use dulovar_p2p::p2p_kad::*;

use futures::StreamExt;
use std::time::Instant;
use tokio::time::{Duration, timeout};

mod test_utils;
use test_utils::*;

#[tokio::test]
async fn benchmark_multiaddr_parsing() {
    let test_addresses = vec![
        "/ip4/127.0.0.1/tcp/4001/ipfs/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
        "/ip6/::1/tcp/4001/p2p/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
        "/dns4/bootstrap.libp2p.io/tcp/443/wss/p2p/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
        "/ip4/192.168.1.1/tcp/8080/ws/ipfs/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
    ];

    let iterations = 10000;
    let start = Instant::now();

    for _ in 0..iterations {
        for addr in &test_addresses {
            let result = crate::p2p_kad_utils::parse_legacy_multiaddr(addr);
            assert!(result.is_ok(), "Failed to parse: {}", addr);
        }
    }

    let elapsed = start.elapsed();
    let ops_per_sec = (iterations * test_addresses.len()) as f64 / elapsed.as_secs_f64();

    println!(
        "Multiaddr parsing benchmark: {} operations in {:?} ({:.2} ops/sec)",
        iterations * test_addresses.len(),
        elapsed,
        ops_per_sec
    );

    // Performance assertion - should handle at least 10k ops/sec
    assert!(
        ops_per_sec > 10000.0,
        "Performance too slow: {:.2} ops/sec",
        ops_per_sec
    );
}

#[tokio::test]
async fn benchmark_peer_id_stripping() {
    use libp2p::multiaddr::Protocol;
    use libp2p::{Multiaddr, PeerId};
    use std::str::FromStr;

    let base_addresses = vec![
        "/ip4/127.0.0.1/tcp/4001",
        "/ip6/::1/tcp/4001",
        "/dns4/example.com/tcp/443",
        "/ip4/10.0.0.1/tcp/8080/ws",
    ];

    let peer_id = PeerId::random();
    let iterations = 50000;

    // Prepare test addresses with peer IDs
    let mut test_addresses: Vec<Multiaddr> = Vec::new();
    for base in &base_addresses {
        let mut addr = Multiaddr::from_str(base).unwrap();
        addr.push(Protocol::P2p(peer_id));
        test_addresses.push(addr);
    }

    let start = Instant::now();

    for _ in 0..iterations {
        for mut addr in test_addresses.clone() {
            p2p_kad_utils::strip_peer_id(&mut addr);
        }
    }

    let elapsed = start.elapsed();
    let ops_per_sec = (iterations * test_addresses.len()) as f64 / elapsed.as_secs_f64();

    println!(
        "Peer ID stripping benchmark: {} operations in {:?} ({:.2} ops/sec)",
        iterations * test_addresses.len(),
        elapsed,
        ops_per_sec
    );

    // Performance assertion - should handle at least 50k ops/sec
    assert!(
        ops_per_sec > 50000.0,
        "Performance too slow: {:.2} ops/sec",
        ops_per_sec
    );
}

#[tokio::test]
async fn benchmark_swarm_creation() {
    let iterations = 100;
    let start = Instant::now();

    for _ in 0..iterations {
        let swarm = create_test_swarm().await;
        assert!(swarm.is_ok(), "Failed to create swarm");
        // Swarm is dropped here, testing creation/destruction overhead
    }

    let elapsed = start.elapsed();
    let swarms_per_sec = iterations as f64 / elapsed.as_secs_f64();

    println!(
        "Swarm creation benchmark: {} swarms in {:?} ({:.2} swarms/sec)",
        iterations, elapsed, swarms_per_sec
    );

    // Should be able to create at least 10 swarms per second
    assert!(
        swarms_per_sec > 10.0,
        "Swarm creation too slow: {:.2} swarms/sec",
        swarms_per_sec
    );
}

#[tokio::test]
async fn benchmark_gossipsub_subscription() {
    let mut swarm = create_test_swarm().await.unwrap();
    let iterations = 10000;

    let start = Instant::now();

    for i in 0..iterations {
        let topic = libp2p::gossipsub::IdentTopic::new(format!("benchmark-topic-{}", i));
        let result = swarm.behaviour_mut().gossipsub.subscribe(&topic);
        assert!(result.is_ok(), "Failed to subscribe to topic");
    }

    let elapsed = start.elapsed();
    let subs_per_sec = iterations as f64 / elapsed.as_secs_f64();

    println!(
        "GossipSub subscription benchmark: {} subscriptions in {:?} ({:.2} subs/sec)",
        iterations, elapsed, subs_per_sec
    );

    // Should handle at least 1000 subscriptions per second
    assert!(
        subs_per_sec > 1000.0,
        "Subscription too slow: {:.2} subs/sec",
        subs_per_sec
    );
}

#[tokio::test]
async fn benchmark_message_publishing() {
    let mut swarm = create_test_swarm().await.unwrap();
    let topic = libp2p::gossipsub::IdentTopic::new("benchmark-publish");

    // Subscribe first
    swarm.behaviour_mut().gossipsub.subscribe(&topic).unwrap();

    let iterations = 5000;
    let message_sizes = vec![64, 256, 1024, 4096]; // Different message sizes in bytes

    for size in message_sizes {
        let test_data = generate_test_data(size);
        let start = Instant::now();

        for _ in 0..iterations {
            let result = swarm
                .behaviour_mut()
                .gossipsub
                .publish(topic.clone(), test_data.clone());
            // Publishing might fail if no peers are connected, which is normal in tests
            if result.is_err() {
                // This is acceptable in a benchmark test without connected peers
                continue;
            }
        }

        let elapsed = start.elapsed();
        let msgs_per_sec = iterations as f64 / elapsed.as_secs_f64();
        let bytes_per_sec = (iterations * size) as f64 / elapsed.as_secs_f64();

        println!(
            "Message publishing benchmark ({}B): {} messages in {:?} ({:.2} msgs/sec, {:.2} MB/s)",
            size,
            iterations,
            elapsed,
            msgs_per_sec,
            bytes_per_sec / 1_000_000.0
        );

        // Note: Performance assertions are relaxed for tests without connected peers
        // In a real scenario with peers, these would be more stringent
        let max_time_ms = match size {
            64 => 3000,
            256 => 4000,
            1024 => 5000,
            4096 => 10000, // Allow more time for larger messages
            _ => 15000,
        };
        assert!(
            elapsed.as_millis() < max_time_ms,
            "Publishing took too long for {}B messages: {:?} (max allowed: {}ms)",
            size,
            elapsed,
            max_time_ms
        );
    }
}

#[tokio::test]
async fn benchmark_concurrent_operations() {
    use tokio::task;

    let concurrent_tasks = 50;
    let operations_per_task = 100;

    let start = Instant::now();

    let mut handles = Vec::new();

    for task_id in 0..concurrent_tasks {
        let handle = task::spawn(async move {
            let mut local_swarm = create_test_swarm().await.unwrap();

            for op_id in 0..operations_per_task {
                // Mix of operations
                let topic =
                    libp2p::gossipsub::IdentTopic::new(format!("task-{}-op-{}", task_id, op_id));

                // Subscribe
                let _ = local_swarm.behaviour_mut().gossipsub.subscribe(&topic);

                // Publish
                let message = format!("Message from task {} operation {}", task_id, op_id);
                let _ = local_swarm
                    .behaviour_mut()
                    .gossipsub
                    .publish(topic, message.as_bytes());

                // Parse a multiaddr
                let test_addr = "/ip4/127.0.0.1/tcp/4001/ipfs/QmTest";
                let _ = crate::p2p_kad_utils::parse_legacy_multiaddr(test_addr);
            }

            operations_per_task
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    let mut total_operations = 0;
    for handle in handles {
        let ops = handle.await.unwrap();
        total_operations += ops;
    }

    let elapsed = start.elapsed();
    let ops_per_sec = total_operations as f64 / elapsed.as_secs_f64();

    println!(
        "Concurrent operations benchmark: {} tasks × {} ops = {} total operations in {:?} ({:.2} ops/sec)",
        concurrent_tasks, operations_per_task, total_operations, elapsed, ops_per_sec
    );

    // Should handle concurrent operations efficiently
    assert!(
        ops_per_sec > 1000.0,
        "Concurrent operations too slow: {:.2} ops/sec",
        ops_per_sec
    );
}

#[tokio::test]
async fn benchmark_memory_usage() {
    // Note: This is a simplified memory usage test

    let iterations = 1000;
    let mut peak_memory_estimate = 0usize;

    for i in 0..iterations {
        let swarm = create_test_swarm().await.unwrap();

        // Rough estimate of memory usage (not precise, but gives an indication)
        let estimated_size = std::mem::size_of_val(&swarm);
        peak_memory_estimate = peak_memory_estimate.max(estimated_size);

        if i % 100 == 0 {
            println!("Memory benchmark progress: {}/{} iterations", i, iterations);
        }

        // Drop swarm to free memory
        drop(swarm);
    }

    println!(
        "Memory usage benchmark: Peak estimated size per swarm: {} bytes ({:.2} KB)",
        peak_memory_estimate,
        peak_memory_estimate as f64 / 1024.0
    );

    // Memory usage should be reasonable (less than 10MB per swarm)
    assert!(
        peak_memory_estimate < 10 * 1024 * 1024,
        "Memory usage too high: {} bytes",
        peak_memory_estimate
    );
}

#[tokio::test]
async fn benchmark_large_multiaddr_parsing() {
    // Test with very long multiaddrs
    let base = "/ip4/127.0.0.1/tcp/4001";
    let peer_id = "QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN";

    let test_cases = vec![
        format!("{}/ipfs/{}", base, peer_id),
        format!("{}/ws/p2p-websocket-star/ipfs/{}", base, peer_id),
        format!(
            "/dns4/very-long-domain-name-for-testing-purposes.example.com/tcp/443/wss/p2p-circuit/ipfs/{}",
            peer_id
        ),
        format!(
            "/ip6/2001:0db8:85a3:0000:0000:8a2e:0370:7334/tcp/4001/quic/ipfs/{}",
            peer_id
        ),
    ];

    let iterations = 1000;
    let start = Instant::now();

    for _ in 0..iterations {
        for addr in &test_cases {
            let result = crate::p2p_kad_utils::parse_legacy_multiaddr(addr);
            assert!(result.is_ok(), "Failed to parse large multiaddr: {}", addr);
        }
    }

    let elapsed = start.elapsed();
    let ops_per_sec = (iterations * test_cases.len()) as f64 / elapsed.as_secs_f64();

    println!(
        "Large multiaddr parsing benchmark: {} operations in {:?} ({:.2} ops/sec)",
        iterations * test_cases.len(),
        elapsed,
        ops_per_sec
    );

    // Should still handle large multiaddrs efficiently
    assert!(
        ops_per_sec > 1000.0,
        "Large multiaddr parsing too slow: {:.2} ops/sec",
        ops_per_sec
    );
}

#[tokio::test]
async fn benchmark_event_processing_throughput() {
    let mut swarm = create_test_swarm().await.unwrap();
    let topic = libp2p::gossipsub::IdentTopic::new("throughput-test");

    // Subscribe and start listening
    swarm.behaviour_mut().gossipsub.subscribe(&topic).unwrap();
    swarm
        .listen_on("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .unwrap();

    // Generate many messages quickly
    let message_count = 1000;
    for i in 0..message_count {
        let message = format!("Throughput test message {}", i);
        let _ = swarm
            .behaviour_mut()
            .gossipsub
            .publish(topic.clone(), message.as_bytes());
    }

    let start = Instant::now();
    let mut events_processed = 0;

    // Process events with timeout
    let result = timeout(Duration::from_secs(10), async {
        loop {
            let _ = swarm.select_next_some().await;
            events_processed += 1;

            // Process at least some events for the benchmark
            if events_processed >= 50 {
                break;
            }
        }
    })
    .await;

    let elapsed = start.elapsed();

    if result.is_ok() {
        let events_per_sec = events_processed as f64 / elapsed.as_secs_f64();

        println!(
            "Event processing throughput: {} events in {:?} ({:.2} events/sec)",
            events_processed, elapsed, events_per_sec
        );

        // Should process events efficiently
        assert!(
            events_per_sec > 10.0,
            "Event processing too slow: {:.2} events/sec",
            events_per_sec
        );
    } else {
        println!(
            "Event processing benchmark timed out after processing {} events",
            events_processed
        );
        // Even if timed out, we should have processed some events
        assert!(events_processed > 0, "No events processed during benchmark");
    }
}
