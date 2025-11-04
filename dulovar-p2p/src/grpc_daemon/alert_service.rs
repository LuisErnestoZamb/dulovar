use futures::{Stream, StreamExt, stream};
use std::pin::Pin;
use tonic::{Request, Response, Status, transport::Server};

use crate::grpc_daemon::alert::{
    AlertConfirmation, AlertRequestData,
    alert_service_server::{AlertService, AlertServiceServer},
};

#[derive(Default)]
pub struct AlertStreamer;

type AlertStream = Pin<Box<dyn Stream<Item = Result<AlertConfirmation, Status>> + Send + 'static>>;

#[tonic::async_trait]
impl AlertService for AlertStreamer {
    type ProcessAndStreamStream = AlertStream;

    async fn process_and_stream(
        &self,
        request: Request<AlertRequestData>,
    ) -> Result<Response<Self::ProcessAndStreamStream>, Status> {
        let req_data = request.into_inner();

        println!("--- gRPC Message ---");
        println!(
            "{} {} (Type: {})",
            req_data.first_name, req_data.last_name, req_data.type_alert
        );
        println!("------------------------------------------\n");

        // Simulación de 5 pasos de confirmación
        let confirmation_messages = vec![
            AlertConfirmation {
                confirmation_id: "CONF-001".into(),
                status_message: "Saved on DB.".into(),
            },
            AlertConfirmation {
                confirmation_id: "CONF-002".into(),
                status_message: "Transmited to Peers.".into(),
            },
        ];

        let output_stream = stream::iter(confirmation_messages.into_iter().map(Ok)).boxed();

        Ok(Response::new(output_stream as Self::ProcessAndStreamStream))
    }
}

// Función para iniciar el servidor, que será llamada desde main.rs
pub async fn run_server(addr: std::net::SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let streamer = AlertStreamer::default();
    println!("gRPC service running on: {}", addr);

    Server::builder()
        .add_service(AlertServiceServer::new(streamer))
        .serve(addr)
        .await?;

    Ok(())
}
