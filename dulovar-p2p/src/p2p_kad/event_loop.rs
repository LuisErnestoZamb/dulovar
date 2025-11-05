use crate::p2p_kad::events::handle_swarm_event;
use crate::p2p_kad::my_behaviour::MyBehaviour;
use futures::StreamExt;
use libp2p::gossipsub;
use std::error::Error;
use tokio::select;
use tokio::sync::mpsc;

pub async fn event_loop(
    receiver: &mut mpsc::UnboundedReceiver<String>,
    swarm: &mut libp2p::Swarm<MyBehaviour>,
    gossipsub_topic: gossipsub::IdentTopic,
) -> Result<(), Box<dyn Error>> {
    loop {
        select! {
            // Receiving message from channel
            message = receiver.recv() => {
                if let Some(msg) = message {
                  // Sending message to peers
                    if let Err(e) = swarm
                        .behaviour_mut()
                        .gossipsub
                        .publish(gossipsub_topic.clone(), msg.as_bytes())
                    {
                        println!("Publish error: {e:?}");
                    }
                }
            },

            // Swarm network event
            event = swarm.select_next_some() => {
                handle_swarm_event(event).await;
            },
        }
    }
}
