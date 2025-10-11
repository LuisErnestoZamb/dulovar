use futures::prelude::*;
use libp2p::{
    Multiaddr, Transport,
    core::transport::upgrade::Version,
    gossipsub, identify,
    multiaddr::Protocol,
    noise, ping,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux,
};

use std::{error::Error, str::FromStr};
use tokio::{io, io::AsyncBufReadExt, select};
use tracing_subscriber::EnvFilter;

pub struct P2pKad {}

impl P2pKad {
    /// for a multiaddr that ends with a peer id, this strips this suffix. Rust-libp2p
    /// only supports dialing to an address without providing the peer id.
    fn strip_peer_id(addr: &mut Multiaddr) {
        let last = addr.pop();
        match last {
            Some(Protocol::P2p(peer_id)) => {
                let mut addr = Multiaddr::empty();
                addr.push(Protocol::P2p(peer_id));
                println!("removing peer id {addr} so this address can be dialed by rust-libp2p");
            }
            Some(other) => addr.push(other),
            _ => {}
        }
    }

    /// parse a legacy multiaddr (replace ipfs with p2p), and strip the peer id
    /// so it can be dialed by rust-libp2p
    fn parse_legacy_multiaddr(text: &str) -> Result<Multiaddr, Box<dyn Error>> {
        let sanitized = text
            .split('/')
            .map(|part| if part == "ipfs" { "p2p" } else { part })
            .collect::<Vec<_>>()
            .join("/");
        let mut res = Multiaddr::from_str(&sanitized)?;
        Self::strip_peer_id(&mut res);
        Ok(res)
    }

    pub async fn init_kad() -> Result<(), Box<dyn Error>> {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::new("info,"))
            .try_init();

        // Create a Gosspipsub topic
        let gossipsub_topic = gossipsub::IdentTopic::new("chat");

        // We create a custom network behaviour that combines gossipsub, ping and identify.
        #[derive(NetworkBehaviour)]
        struct MyBehaviour {
            gossipsub: gossipsub::Behaviour,
            identify: identify::Behaviour,
            ping: ping::Behaviour,
        }

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

        // Reach out to other nodes if specified
        for to_dial in std::env::args().skip(1) {
            let addr: Multiaddr = Self::parse_legacy_multiaddr(&to_dial)?;
            swarm.dial(addr)?;
            println!("Dialed {to_dial:?}")
        }

        // Read full lines from stdin
        let mut stdin = io::BufReader::new(io::stdin()).lines();

        // Listen on all interfaces and whatever port the OS assigns
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

        // Kick it off
        loop {
            select! {
                Ok(Some(line)) = stdin.next_line() => {
                    if let Err(e) = swarm
                        .behaviour_mut()
                        .gossipsub
                        .publish(gossipsub_topic.clone(), line.as_bytes())
                    {
                        println!("Publish error: {e:?}");
                    }
                },
                event = swarm.select_next_some() => {
                    match event {
                        SwarmEvent::NewListenAddr { address, .. } => {
                            println!("Listening on {address:?}");
                        }
                        SwarmEvent::Behaviour(MyBehaviourEvent::Identify(event)) => {
                            println!("identify: {event:?}");
                        }
                        SwarmEvent::Behaviour(MyBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                            propagation_source: peer_id,
                            message_id: id,
                            message,
                        })) => {
                            println!(
                                "Got message: {} with id: {} from peer: {:?}",
                                String::from_utf8_lossy(&message.data),
                                id,
                                peer_id
                            )
                        }
                        SwarmEvent::Behaviour(MyBehaviourEvent::Ping(event)) => {
                            match event {
                                ping::Event {
                                    peer,
                                    result: Result::Ok(rtt),
                                    ..
                                } => {
                                    println!(
                                        "ping: rtt to {} is {} ms",
                                        peer.to_base58(),
                                        rtt.as_millis()
                                    );
                                }
                                ping::Event {
                                    peer,
                                    result: Result::Err(ping::Failure::Timeout),
                                    ..
                                } => {
                                    println!("ping: timeout to {}", peer.to_base58());
                                }
                                ping::Event {
                                    peer,
                                    result: Result::Err(ping::Failure::Unsupported),
                                    ..
                                } => {
                                    println!("ping: {} does not support ping protocol", peer.to_base58());
                                }
                                ping::Event {
                                    peer,
                                    result: Result::Err(ping::Failure::Other { error }),
                                    ..
                                } => {
                                    println!("ping: ping::Failure with {}: {error}", peer.to_base58());
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
