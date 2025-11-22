pub mod event_loop;
pub mod events;
pub mod keypair_file;
pub mod my_behaviour;
pub mod p2p_kad_utils;
pub mod rest_request;
pub mod swarm_builder;

use crate::config::Configuration;
use crate::p2p_kad::event_loop::event_loop;
use crate::p2p_kad::keypair_file::read_keypair_file;
use crate::p2p_kad::p2p_kad_utils::add_new_nodes;
use crate::p2p_kad::rest_request::RestRequest;
use libp2p::gossipsub;
use std::error::Error;
use tokio::sync::mpsc;

pub struct P2pKad {
    receiver: mpsc::UnboundedReceiver<String>,
}

impl P2pKad {
    pub fn new(receiver: mpsc::UnboundedReceiver<String>) -> Self {
        Self { receiver }
    }

    pub async fn run(mut self, c: Configuration) -> Result<(), Box<dyn Error>> {
        let nodes = RestRequest::new(c.clone());
        nodes.register_node().await?;

        let id_keys = read_keypair_file(c.private_key_file)?;
        let mut swarm = swarm_builder::swarm_kad(id_keys).unwrap();

        // Create a Gosspipsub topic
        let gossipsub_topic = gossipsub::IdentTopic::new("operations");
        println!("Subscribing to {gossipsub_topic:?}");
        swarm
            .behaviour_mut()
            .gossipsub
            .subscribe(&gossipsub_topic)
            .unwrap();

        add_new_nodes(&mut swarm, nodes.get_nodes().await?).unwrap();
        // Listen on all interfaces and whatever port the OS assigns
        swarm.listen_on(format!("/ip4/0.0.0.0/tcp/{}", c.p2p_port).parse()?)?;

        event_loop(&mut self.receiver, &mut swarm, gossipsub_topic).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;
    use tokio;

    const JSON_RESPONSE: &str = r#"{"address":"0.0.0.0:50001","valid":1,"master":0}"#;

    async fn setup_mock_get_nodes() {
        let mut server = Server::new_async().await;

        server
            .mock("GET", "/test")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(JSON_RESPONSE)
            .create_async()
            .await;
    }

    async fn setup_mock_public_ip() {
        let mut server = Server::new_async().await;
        server
            .mock("GET", "?format=text")
            .with_status(200)
            .with_body("0.0.0.0")
            .create_async()
            .await;
    }

    fn port_struct(p2p_port: u16) -> Configuration {
        Configuration {
            grpc_port: 50001,
            p2p_port,
            production: false,
            private_key_file: "p2p_private_key.bin".to_string(),
            node_web_server_url: "http://0.0.0.0".to_string(),
            public_ip_url_1: "http://0.0.0.0".to_string(),
            public_ip_url_2: "http://0.0.0.0".to_string(),
        }
    }

    #[tokio::test]
    async fn test_p2p_ping() {
        setup_mock_get_nodes().await;
        setup_mock_public_ip().await;

        let (_sender_a, receiver_a) = mpsc::unbounded_channel();
        let (_sender_b, receiver_b) = mpsc::unbounded_channel();
        let p2p_kad_a = P2pKad::new(receiver_a);
        let p2p_kad_b = P2pKad::new(receiver_b);
        let _p2p_a = p2p_kad_a.run(port_struct(50001)).await;
        let _p2p_b = p2p_kad_b.run(port_struct(50002)).await;
    }
}
