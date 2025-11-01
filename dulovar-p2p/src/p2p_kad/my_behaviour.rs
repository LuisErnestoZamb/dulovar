use libp2p::swarm::NetworkBehaviour;
use libp2p::{gossipsub, identify, ping};

#[derive(NetworkBehaviour)]
pub struct MyBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub identify: identify::Behaviour,
    pub ping: ping::Behaviour,
}
