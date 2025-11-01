use futures::prelude::*;
use libp2p::{
    Multiaddr, PeerId, Transport,
    core::transport::upgrade::Version,
    gossipsub, identify,
    multiaddr::Protocol,
    noise, ping,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux,
};
use std::{error::Error, time::Duration};
use tokio::time::timeout;

/// Test behaviour combining gossipsub, identify, and ping protocols
#[derive(NetworkBehaviour)]
pub struct TestBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub identify: identify::Behaviour,
    pub ping: ping::Behaviour,
}

/// Configuration for test swarms
pub struct TestSwarmConfig {
    pub protocol_version: String,
    pub max_transmit_size: usize,
    pub enable_mdns: bool,
}

impl Default for TestSwarmConfig {
    fn default() -> Self {
        Self {
            protocol_version: "/test/1.0.0".to_string(),
            max_transmit_size: 262144,
            enable_mdns: false,
        }
    }
}

/// Creates a test swarm with the given configuration
pub async fn create_test_swarm_with_config(
    config: TestSwarmConfig,
) -> Result<libp2p::Swarm<TestBehaviour>, Box<dyn Error + Send + Sync>> {
    let swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_other_transport(|key| {
            let noise_config = noise::Config::new(key).unwrap();
            let yamux_config = yamux::Config::default();

            let base_transport = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true));
            base_transport
                .upgrade(Version::V1Lazy)
                .authenticate(noise_config)
                .multiplex(yamux_config)
        })?
        .with_behaviour(|key| {
            let gossipsub_config = gossipsub::ConfigBuilder::default()
                .max_transmit_size(config.max_transmit_size)
                .build()
                .unwrap();

            Ok(TestBehaviour {
                gossipsub: gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossipsub_config,
                )
                .expect("Valid configuration"),
                identify: identify::Behaviour::new(identify::Config::new(
                    config.protocol_version,
                    key.public(),
                )),
                ping: ping::Behaviour::new(ping::Config::new()),
            })
        })?
        .build();

    Ok(swarm)
}

/// Creates a basic test swarm with default configuration
pub async fn create_test_swarm()
-> Result<libp2p::Swarm<TestBehaviour>, Box<dyn Error + Send + Sync>> {
    create_test_swarm_with_config(TestSwarmConfig::default()).await
}

/// Waits for a swarm to start listening and returns the listening address
pub async fn wait_for_listen_addr(
    swarm: &mut libp2p::Swarm<TestBehaviour>,
    timeout_secs: u64,
) -> Result<Multiaddr, Box<dyn Error + Send + Sync>> {
    let result = timeout(Duration::from_secs(timeout_secs), async {
        loop {
            if let SwarmEvent::NewListenAddr { address, .. } = swarm.select_next_some().await {
                return address;
            }
        }
    })
    .await;

    match result {
        Ok(addr) => Ok(addr),
        Err(_) => Err("Timeout waiting for listen address".into()),
    }
}

/// Creates a test peer ID for testing purposes
pub fn create_test_peer_id() -> PeerId {
    PeerId::random()
}

/// Creates a test multiaddr with a random port
pub fn create_test_multiaddr() -> Multiaddr {
    "/ip4/127.0.0.1/tcp/0".parse().unwrap()
}

/// Creates a test multiaddr with a specific peer ID
pub fn create_test_multiaddr_with_peer(peer_id: PeerId) -> Multiaddr {
    let mut addr = create_test_multiaddr();
    addr.push(Protocol::P2p(peer_id));
    addr
}

/// Test message structure for gossipsub testing
#[derive(Debug, Clone, PartialEq)]
pub struct TestMessage {
    pub content: Vec<u8>,
    pub topic: String,
    pub sender: Option<PeerId>,
}

impl TestMessage {
    pub fn new(content: &str, topic: &str) -> Self {
        Self {
            content: content.as_bytes().to_vec(),
            topic: topic.to_string(),
            sender: None,
        }
    }

    pub fn with_sender(mut self, sender: PeerId) -> Self {
        self.sender = Some(sender);
        self
    }
}

/// Helper to collect events from a swarm for testing
pub struct EventCollector {
    pub listen_addrs: Vec<Multiaddr>,
    pub gossipsub_messages: Vec<TestMessage>,
    pub identify_events: Vec<identify::Event>,
    pub ping_events: Vec<ping::Event>,
}

impl EventCollector {
    pub fn new() -> Self {
        Self {
            listen_addrs: Vec::new(),
            gossipsub_messages: Vec::new(),
            identify_events: Vec::new(),
            ping_events: Vec::new(),
        }
    }

    pub fn handle_event(&mut self, event: SwarmEvent<TestBehaviourEvent>) {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                self.listen_addrs.push(address);
            }
            SwarmEvent::Behaviour(TestBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                propagation_source: peer_id,
                message,
                ..
            })) => {
                let test_msg = TestMessage {
                    content: message.data,
                    topic: message.topic.to_string(),
                    sender: Some(peer_id),
                };
                self.gossipsub_messages.push(test_msg);
            }
            SwarmEvent::Behaviour(TestBehaviourEvent::Identify(event)) => {
                self.identify_events.push(event);
            }
            SwarmEvent::Behaviour(TestBehaviourEvent::Ping(event)) => {
                self.ping_events.push(event);
            }
            _ => {}
        }
    }

    pub fn message_count(&self) -> usize {
        self.gossipsub_messages.len()
    }

    pub fn has_message_with_content(&self, content: &str) -> bool {
        self.gossipsub_messages
            .iter()
            .any(|msg| msg.content == content.as_bytes())
    }
}

/// Runs a swarm for a specified duration and collects events
pub async fn run_swarm_with_timeout(
    mut swarm: libp2p::Swarm<TestBehaviour>,
    duration_secs: u64,
) -> Result<EventCollector, Box<dyn Error + Send + Sync>> {
    let mut collector = EventCollector::new();

    let result = timeout(Duration::from_secs(duration_secs), async {
        loop {
            let event = swarm.select_next_some().await;
            collector.handle_event(event);
        }
    })
    .await;

    // Timeout is expected - we just want to collect events during the specified duration
    match result {
        Err(_) => Ok(collector), // Timeout is expected
        Ok(_) => Ok(collector),  // Unlikely but handle gracefully
    }
}

/// Creates two connected test swarms for peer-to-peer testing
pub async fn create_connected_swarms() -> Result<
    (
        libp2p::Swarm<TestBehaviour>,
        libp2p::Swarm<TestBehaviour>,
        Multiaddr,
    ),
    Box<dyn Error + Send + Sync>,
> {
    let mut swarm1 = create_test_swarm().await?;
    let mut swarm2 = create_test_swarm().await?;

    // Start listening on swarm1
    swarm1.listen_on(create_test_multiaddr())?;
    let listen_addr = wait_for_listen_addr(&mut swarm1, 5).await?;

    // Connect swarm2 to swarm1
    swarm2.dial(listen_addr.clone())?;

    Ok((swarm1, swarm2, listen_addr))
}

/// Validates that a multiaddr has the expected format
pub fn validate_multiaddr_format(addr: &Multiaddr, expected_protocols: &[&str]) -> bool {
    let addr_string = addr.to_string();
    expected_protocols
        .iter()
        .all(|protocol| addr_string.contains(protocol))
}

/// Generates test data for performance testing
pub fn generate_test_data(size_bytes: usize) -> Vec<u8> {
    (0..size_bytes).map(|i| (i % 256) as u8).collect()
}

/// Test constants for consistent testing
pub mod constants {
    pub const DEFAULT_TIMEOUT_SECS: u64 = 5;
    pub const DEFAULT_TEST_TOPIC: &str = "test-topic";
    pub const PERFORMANCE_TEST_ITERATIONS: usize = 100;
    pub const LARGE_MESSAGE_SIZE: usize = 1024 * 64; // 64KB
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_test_swarm() {
        let swarm = create_test_swarm().await;
        assert!(swarm.is_ok());
    }

    #[test]
    fn test_create_test_peer_id() {
        let peer_id1 = create_test_peer_id();
        let peer_id2 = create_test_peer_id();
        assert_ne!(peer_id1, peer_id2);
    }

    #[test]
    fn test_create_test_multiaddr() {
        let addr = create_test_multiaddr();
        assert!(addr.to_string().starts_with("/ip4/127.0.0.1/tcp/"));
    }

    #[test]
    fn test_test_message_creation() {
        let msg = TestMessage::new("hello", "test-topic");
        assert_eq!(msg.content, b"hello");
        assert_eq!(msg.topic, "test-topic");
        assert!(msg.sender.is_none());
    }

    #[test]
    fn test_event_collector() {
        let collector = EventCollector::new();
        assert_eq!(collector.message_count(), 0);
        assert!(!collector.has_message_with_content("test"));
    }

    #[test]
    fn test_validate_multiaddr_format() {
        let addr = "/ip4/127.0.0.1/tcp/1234".parse::<Multiaddr>().unwrap();
        assert!(validate_multiaddr_format(&addr, &["ip4", "tcp"]));
        assert!(!validate_multiaddr_format(&addr, &["ip6", "udp"]));
    }

    #[test]
    fn test_generate_test_data() {
        let data = generate_test_data(300);
        assert_eq!(data.len(), 300);
        assert_eq!(data[0], 0);
        assert_eq!(data[255], 255);
        assert_eq!(data[256], 0); // Wraps around
    }
}
