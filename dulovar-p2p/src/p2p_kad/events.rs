use crate::p2p_kad::my_behaviour::MyBehaviourEvent;
use libp2p::{gossipsub, ping, swarm::SwarmEvent};

pub async fn handle_swarm_event(event: SwarmEvent<MyBehaviourEvent>) {
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
            );
        }
        SwarmEvent::Behaviour(MyBehaviourEvent::Ping(event)) => match event {
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
        },
        _ => {}
    }
}
