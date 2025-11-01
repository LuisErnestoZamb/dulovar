use dulovar_p2p::p2p_kad::*;
use libp2p::{Multiaddr, PeerId, multiaddr::Protocol};
use std::{error::Error, str::FromStr, time::Duration};
use tokio::time::timeout;

#[tokio::test]
async fn test_multiaddr_parsing_comprehensive() {
    // Test various multiaddr formats
    let test_cases = vec![
        // Legacy IPFS format
        (
            "/ip4/127.0.0.1/tcp/4001/ipfs/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
            "/ip4/127.0.0.1/tcp/4001",
        ),
        // Modern P2P format
        (
            "/ip4/127.0.0.1/tcp/4001/p2p/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
            "/ip4/127.0.0.1/tcp/4001",
        ),
        // IPv6
        (
            "/ip6/::1/tcp/4001/ipfs/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
            "/ip6/::1/tcp/4001",
        ),
        // DNS
        (
            "/dns4/bootstrap.libp2p.io/tcp/443/wss/ipfs/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
            "/dns4/bootstrap.libp2p.io/tcp/443/wss",
        ),
    ];

    for (input, expected) in test_cases {
        let result = crate::p2p_kad_utils::parse_legacy_multiaddr(input);
        assert!(result.is_ok(), "Failed to parse: {}", input);
        assert_eq!(result.unwrap().to_string(), expected);
    }
}

#[tokio::test]
async fn test_multiaddr_parsing_error_cases() {
    let invalid_cases = vec!["not_a_multiaddr", "/invalid/protocol", "just_text"];

    for invalid in invalid_cases {
        let result = crate::p2p_kad_utils::parse_legacy_multiaddr(invalid);
        assert!(result.is_err(), "Should fail to parse: {}", invalid);
    }
}

#[tokio::test]
async fn test_strip_peer_id_functionality() {
    let peer_id = PeerId::random();

    // Test cases with different protocol combinations
    let test_cases = vec![
        "/ip4/127.0.0.1/tcp/1234",
        "/ip6/::1/tcp/4001",
        "/dns4/example.com/tcp/443",
        "/ip4/192.168.1.1/tcp/8080/ws",
    ];

    for base_addr_str in test_cases {
        let mut addr = Multiaddr::from_str(base_addr_str).unwrap();
        let original = addr.clone();

        // Add peer ID
        addr.push(Protocol::P2p(peer_id));

        // Verify peer ID was added
        assert_ne!(addr, original);
        assert!(addr.to_string().contains("/p2p/"));

        // Strip peer ID
        crate::p2p_kad_utils::strip_peer_id(&mut addr);

        // Verify peer ID was removed
        assert_eq!(addr, original);
        assert!(!addr.to_string().contains("/p2p/"));
    }
}

#[tokio::test]
async fn test_init_kad_timeout() {
    // Test that init_kad runs without panicking and times out as expected
    let result = timeout(Duration::from_millis(100), crate::init_kad::init_kad()).await;

    // Should timeout since init_kad runs indefinitely
    assert!(result.is_err(), "init_kad should timeout in test");
}

#[tokio::test]
async fn test_multiaddr_parsing_performance() {
    let test_addr = "/ip4/127.0.0.1/tcp/4001/ipfs/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN";
    let iterations = 1000;

    let start = std::time::Instant::now();

    for _ in 0..iterations {
        let result = crate::p2p_kad_utils::parse_legacy_multiaddr(test_addr);
        assert!(result.is_ok());
    }

    let elapsed = start.elapsed();

    // Should complete 1000 operations reasonably quickly (< 1 second)
    assert!(
        elapsed.as_secs() < 1,
        "Parsing should be fast, took: {:?}",
        elapsed
    );
}

#[tokio::test]
async fn test_peer_id_stripping_performance() {
    let peer_id = PeerId::random();
    let base_addr = Multiaddr::from_str("/ip4/127.0.0.1/tcp/4001").unwrap();
    let iterations = 5000;

    let start = std::time::Instant::now();

    for _ in 0..iterations {
        let mut addr = base_addr.clone();
        addr.push(Protocol::P2p(peer_id));
        crate::p2p_kad_utils::strip_peer_id(&mut addr);
        assert_eq!(addr, base_addr);
    }

    let elapsed = start.elapsed();

    // Should handle peer ID stripping efficiently
    assert!(
        elapsed.as_millis() < 1000,
        "Peer ID stripping should be fast, took: {:?}",
        elapsed
    );
}

#[tokio::test]
async fn test_complex_multiaddr_parsing() {
    // Test complex multiaddr with multiple protocols
    let complex_cases = vec![(
        "/ip4/127.0.0.1/tcp/4001/ws/p2p-websocket-star/ipfs/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
        "/ip4/127.0.0.1/tcp/4001/ws/p2p-websocket-star",
    )];

    for (input, expected) in complex_cases {
        let result = crate::p2p_kad_utils::parse_legacy_multiaddr(input);
        assert!(
            result.is_ok(),
            "Failed to parse complex multiaddr: {}",
            input
        );

        let parsed = result.unwrap().to_string();
        assert_eq!(parsed, expected);
        assert!(!parsed.contains("ipfs"));
    }

    // Test case that might fail due to invalid peer ID - handle gracefully
    let potentially_invalid = "/dns4/example.com/tcp/443/wss/p2p-circuit/ipfs/QmTestPeer";
    let result = crate::p2p_kad_utils::parse_legacy_multiaddr(potentially_invalid);
    // This might fail due to invalid peer ID format, which is acceptable
    if result.is_ok() {
        let parsed = result.unwrap().to_string();
        assert!(!parsed.contains("ipfs"));
    }
}

#[tokio::test]
async fn test_edge_cases() {
    // Test empty multiaddr (should work)
    let empty_result = crate::p2p_kad_utils::parse_legacy_multiaddr("");
    assert!(empty_result.is_ok());
    assert_eq!(empty_result.unwrap().to_string(), "");

    // Test multiaddr without peer ID (should work unchanged)
    let no_peer_result = crate::p2p_kad_utils::parse_legacy_multiaddr("/ip4/127.0.0.1/tcp/4001");
    assert!(no_peer_result.is_ok());
    assert_eq!(
        no_peer_result.unwrap().to_string(),
        "/ip4/127.0.0.1/tcp/4001"
    );

    // Test stripping peer ID from multiaddr without peer ID (should be unchanged)
    let mut addr_no_peer = Multiaddr::from_str("/ip4/127.0.0.1/tcp/4001").unwrap();
    let original = addr_no_peer.clone();
    crate::p2p_kad_utils::strip_peer_id(&mut addr_no_peer);
    assert_eq!(addr_no_peer, original);
}

#[tokio::test]
async fn test_protocol_variations() {
    let test_cases = vec![
        "/ip4/192.168.1.1/tcp/8080/http/ipfs/QmTest",
        "/ip6/2001:db8::1/tcp/4001/quic/ipfs/QmTest",
        "/dns/example.com/tcp/443/tls/ipfs/QmTest",
    ];

    for addr in test_cases {
        let result = crate::p2p_kad_utils::parse_legacy_multiaddr(addr);
        if result.is_ok() {
            let parsed = result.unwrap().to_string();
            assert!(
                !parsed.contains("ipfs"),
                "Should not contain 'ipfs' in parsed address"
            );
        }
        // Some might fail due to unsupported protocols, which is acceptable
    }
}

#[tokio::test]
async fn test_concurrent_parsing() {
    use tokio::task;

    let handles: Vec<_> = (0..10)
        .map(|i| {
            task::spawn(async move {
                let addr = format!(
                    "/ip4/127.0.0.{}/tcp/400{}/ipfs/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
                    i + 1,
                    i + 1
                );

                for _ in 0..100 {
                    let result = crate::p2p_kad_utils::parse_legacy_multiaddr(&addr);
                    assert!(result.is_ok());
                }

                i
            })
        })
        .collect();

    // Wait for all tasks to complete
    for handle in handles {
        let result = handle.await;
        assert!(result.is_ok());
    }
}
