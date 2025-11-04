pub mod alert;
pub mod alert_service;
use tokio::sync::mpsc;
use tonic::transport::Server;

use crate::grpc_daemon::{
    alert::alert_service_server::AlertServiceServer, alert_service::AlertStreamer,
};

pub struct GrpcDaemon {
    sender: mpsc::UnboundedSender<String>,
}

impl GrpcDaemon {
    pub fn new(sender: mpsc::UnboundedSender<String>) -> Self {
        Self { sender }
    }

    pub async fn run_server(
        &self,
        addr: std::net::SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let streamer = AlertStreamer::default();
        println!("gRPC service running on: {}", addr);

        Server::builder()
            .add_service(AlertServiceServer::new(streamer))
            .serve(addr)
            .await?;

        Ok(())
    }
}
