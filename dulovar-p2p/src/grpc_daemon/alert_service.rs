use futures::{Stream, StreamExt, stream};
use std::pin::Pin;
use tokio::sync::mpsc;
use tonic::{Request, Response, Status};

use crate::grpc_daemon::alert::{
    AlertConfirmation, AlertRequestData, alert_service_server::AlertService,
};

pub struct AlertStreamer {
    sender: mpsc::UnboundedSender<String>,
}

impl AlertStreamer {
    pub fn new(sender: mpsc::UnboundedSender<String>) -> Self {
        Self { sender }
    }
}

type AlertStream = Pin<Box<dyn Stream<Item = Result<AlertConfirmation, Status>> + Send + 'static>>;

#[tonic::async_trait]
impl AlertService for AlertStreamer {
    type ProcessAndStreamStream = AlertStream;

    async fn process_and_stream(
        &self,
        request: Request<AlertRequestData>,
    ) -> Result<Response<Self::ProcessAndStreamStream>, Status> {
        let req_data = request.into_inner();

        // Send data through channel
        if let Err(e) = self.sender.send(req_data.type_alert.to_string()) {
            return Err(Status::internal("Failed to process request"));
        }

        println!("------ gRPC Message -------");
        println!(
            "{} {} (Type: {})",
            req_data.first_name, req_data.last_name, req_data.type_alert
        );
        println!("------ gRPC Message end ---\n");

        let confirmation_messages = vec![
            AlertConfirmation {
                confirmation_id: "CONF-001".into(),
                status_message: "Saved on DB.".into(),
            },
            AlertConfirmation {
                confirmation_id: "CONF-002".into(),
                status_message: "Transmitted to Peers.".into(),
            },
        ];

        let output_stream = stream::iter(confirmation_messages.into_iter().map(Ok))
            .inspect(|result| {
                if let Ok(conf) = result {
                    println!("-> Sending confirmation: {}", conf.status_message);
                }
            })
            .boxed();
        Ok(Response::new(output_stream as Self::ProcessAndStreamStream))
    }
}
