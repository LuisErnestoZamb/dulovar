use crate::p2p_kad::my_behaviour::MyBehaviour;
use libp2p::{
    Swarm, Transport, core::transport::upgrade::Version, gossipsub, identify, identity::Keypair,
    noise, ping, tcp, yamux,
};
use std::error::Error;
use tokio::io;

pub fn swarm_kad(
    id_keys: Keypair,
) -> Result<Swarm<MyBehaviour>, std::boxed::Box<dyn Error + 'static>> {
    Ok(
        libp2p::SwarmBuilder::with_existing_identity(id_keys.clone())
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
            .build(),
    )
}
