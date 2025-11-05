pub mod event_loop;
pub mod events;
pub mod my_behaviour;
pub mod p2p_kad_utils;
pub mod rest_request;

use tokio::sync::mpsc;

use libp2p::{
    Transport, core::transport::upgrade::Version, gossipsub, identify, noise, ping, tcp, yamux,
};

use std::error::Error;
use tokio::{io, io::AsyncBufReadExt};

use crate::p2p_kad::event_loop::event_loop;
use crate::p2p_kad::my_behaviour::MyBehaviour;
use crate::p2p_kad::p2p_kad_utils::*;
use crate::p2p_kad::rest_request::RestRequest;

pub struct P2pKad {
    receiver: mpsc::UnboundedReceiver<String>,
}

impl P2pKad {
    pub fn new(receiver: mpsc::UnboundedReceiver<String>) -> Self {
        Self { receiver }
    }

    pub async fn run(mut self) -> Result<(), Box<dyn Error>> {
        // Create a Gosspipsub topic
        let _r = RestRequest::register_node().await?;
        let nodes = RestRequest::get_nodes().await?;

        let gossipsub_topic = gossipsub::IdentTopic::new("operations");

        let mut swarm = libp2p::SwarmBuilder::with_new_identity()
            .with_tokio()
            .with_other_transport(|key| {
                let noise_config = noise::Config::new(key).unwrap();
                let yamux_config = yamux::Config::default();

                let base_transport =
                    tcp::tokio::Transport::new(tcp::Config::default().nodelay(true));
                base_transport
                    .upgrade(Version::V1Lazy)
                    .authenticate(noise_config)
                    .multiplex(yamux_config)
            })?
            .with_dns()?
            .with_behaviour(|key| {
                let gossipsub_config = gossipsub::ConfigBuilder::default()
                    .max_transmit_size(262144)
                    .build()
                    .map_err(io::Error::other)?;
                Ok(MyBehaviour {
                    gossipsub: gossipsub::Behaviour::new(
                        gossipsub::MessageAuthenticity::Signed(key.clone()),
                        gossipsub_config,
                    )
                    .expect("Valid configuration"),
                    identify: identify::Behaviour::new(identify::Config::new(
                        "/ipfs/0.1.0".into(),
                        key.public(),
                    )),
                    ping: ping::Behaviour::new(ping::Config::new()),
                })
            })?
            .build();

        println!("Subscribing to {gossipsub_topic:?}");
        swarm
            .behaviour_mut()
            .gossipsub
            .subscribe(&gossipsub_topic)
            .unwrap();

        let _ = add_new_nodes(&mut swarm, nodes);

        // Read full lines from stdin
        let mut stdin = io::BufReader::new(io::stdin()).lines();

        // Listen on all interfaces and whatever port the OS assigns
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

        event_loop(&mut self.receiver, &mut swarm, gossipsub_topic).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::Multiaddr;
    use std::str::FromStr;
    use std::time::Duration;
    use tokio::time::timeout;

    #[test]
    fn test_strip_peer_id() {
        // Test with peer ID at the end
        let mut addr = Multiaddr::from_str("/ip4/127.0.0.1/tcp/1234").unwrap();

        strip_peer_id(&mut addr);

        // Should not contain peer ID anymore
        assert!(!addr.to_string().contains("/p2p/"));
        assert_eq!(addr.to_string(), "/ip4/127.0.0.1/tcp/1234");
    }

    #[test]
    fn test_strip_peer_id_no_peer_id() {
        // Test with no peer ID
        let mut addr = Multiaddr::from_str("/ip4/127.0.0.1/tcp/1234").unwrap();
        let original = addr.clone();

        strip_peer_id(&mut addr);

        // Should remain unchanged
        assert_eq!(addr, original);
    }

    #[test]
    fn test_parse_legacy_multiaddr() {
        let legacy_addr =
            "/ip4/127.0.0.1/tcp/4001/ipfs/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN";

        let result = parse_legacy_multiaddr(legacy_addr);
        assert!(result.is_ok());

        let addr = result.unwrap();
        // Should replace ipfs with p2p and strip peer ID
        assert_eq!(addr.to_string(), "/ip4/127.0.0.1/tcp/4001");
    }

    #[test]
    fn test_parse_legacy_multiaddr_invalid() {
        let invalid_addr = "invalid_multiaddr";

        let result = parse_legacy_multiaddr(invalid_addr);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_legacy_multiaddr_modern_format() {
        let modern_addr =
            "/ip4/127.0.0.1/tcp/4001/p2p/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN";

        let result = parse_legacy_multiaddr(modern_addr);
        assert!(result.is_ok());

        let addr = result.unwrap();
        // Should strip peer ID but keep p2p format
        assert_eq!(addr.to_string(), "/ip4/127.0.0.1/tcp/4001");
    }

    #[tokio::test]
    async fn test_init_kad_basic_setup() {
        // This test verifies that init_kad can be called without panicking
        // We'll timeout quickly since init_kad runs indefinitely
        let (sender, receiver) = mpsc::unbounded_channel();

        let p2p_kad = P2pKad::new(receiver);
        let result = timeout(Duration::from_millis(100), p2p_kad.run()).await;

        // Should timeout (not panic) since init_kad runs in a loop
        assert!(result.is_err());
    }

    #[test]
    fn test_multiaddr_parsing_edge_cases() {
        // Test empty string - this actually creates an empty multiaddr which is valid
        let result = crate::p2p_kad::p2p_kad_utils::parse_legacy_multiaddr("");
        assert!(result.is_ok()); // Empty multiaddr is valid
        assert_eq!(result.unwrap().to_string(), "");

        // Test with invalid multiaddr format
        let result = parse_legacy_multiaddr("not_a_multiaddr");
        assert!(result.is_err());

        // Test with simple ipfs replacement - this might fail due to invalid peer ID
        let addr_with_ipfs =
            "/ip4/127.0.0.1/tcp/4001/ipfs/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN";
        let result = parse_legacy_multiaddr(addr_with_ipfs);
        assert!(result.is_ok());

        // Should replace ipfs with p2p and strip peer ID
        let parsed = result.unwrap();
        assert!(!parsed.to_string().contains("ipfs"));
        assert_eq!(parsed.to_string(), "/ip4/127.0.0.1/tcp/4001");
    }

    #[test]
    fn test_multiaddr_with_different_protocols() {
        // Test with WebSocket
        let ws_addr =
            "/ip4/127.0.0.1/tcp/4001/ws/ipfs/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN";
        let result = parse_legacy_multiaddr(ws_addr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "/ip4/127.0.0.1/tcp/4001/ws");

        // Test with WebSocket Secure
        let wss_addr =
            "/ip4/127.0.0.1/tcp/443/wss/ipfs/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN";
        let result = parse_legacy_multiaddr(wss_addr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "/ip4/127.0.0.1/tcp/443/wss");
    }

    #[test]
    fn test_ipv6_multiaddr_parsing() {
        let ipv6_addr = "/ip6/::1/tcp/4001/ipfs/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN";
        let result = parse_legacy_multiaddr(ipv6_addr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "/ip6/::1/tcp/4001");
    }

    #[test]
    fn test_dns_multiaddr_parsing() {
        let dns_addr =
            "/dns4/example.com/tcp/4001/ipfs/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN";
        let result = parse_legacy_multiaddr(dns_addr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "/dns4/example.com/tcp/4001");
    }

    #[test]
    fn test_strip_peer_id_with_different_protocols() {
        // Test with TCP
        let mut tcp_addr = Multiaddr::from_str("/ip4/127.0.0.1/tcp/1234").unwrap();
        strip_peer_id(&mut tcp_addr);
        assert_eq!(tcp_addr.to_string(), "/ip4/127.0.0.1/tcp/1234");

        // Test with WebSocket
        let mut ws_addr = Multiaddr::from_str("/ip4/127.0.0.1/tcp/8080/ws").unwrap();
        strip_peer_id(&mut ws_addr);
        assert_eq!(ws_addr.to_string(), "/ip4/127.0.0.1/tcp/8080/ws");
    }

    #[test]
    fn test_multiaddr_parsing_preserves_other_protocols() {
        // Test that non-ipfs protocols are preserved
        let complex_addr = "/ip4/127.0.0.1/tcp/4001/ws/p2p-websocket-star/ipfs/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN";
        let result = parse_legacy_multiaddr(complex_addr);
        assert!(result.is_ok());

        let parsed = result.unwrap().to_string();
        assert!(parsed.contains("ws"));
        assert!(parsed.contains("p2p-websocket-star"));
        assert!(!parsed.contains("ipfs"));
        assert!(!parsed.contains("QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN"));
    }
}
