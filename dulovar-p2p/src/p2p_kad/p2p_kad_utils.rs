use libp2p::Swarm;
use libp2p::{Multiaddr, multiaddr::Protocol};

use std::{error::Error, str::FromStr};

/// for a multiaddr that ends with a peer id, this strips this suffix. Rust-libp2p
/// only supports dialing to an address without providing the peer id.
pub fn strip_peer_id(addr: &mut Multiaddr) {
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
pub fn parse_legacy_multiaddr(text: &str) -> Result<Multiaddr, Box<dyn Error>> {
    let sanitized = text
        .split('/')
        .map(|part| if part == "ipfs" { "p2p" } else { part })
        .collect::<Vec<_>>()
        .join("/");
    let mut res = Multiaddr::from_str(&sanitized)?;
    strip_peer_id(&mut res);
    Ok(res)
}

pub fn add_new_nodes(
    swarm: &mut Swarm<impl libp2p::swarm::NetworkBehaviour>,
    nodes: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    for to_dial in nodes {
        println!("Dialed {to_dial:?}");
        if let Ok(addr) = parse_legacy_multiaddr(&to_dial) {
            swarm.dial(addr)?;
        }
    }
    Ok(())
}
