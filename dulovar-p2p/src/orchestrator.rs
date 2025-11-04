use dulovar_p2p::grpc_daemon::GrpcDaemon;
use dulovar_p2p::p2p_kad::P2pKad;
use std::error::Error;
use tokio;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

/// Runs the Kademlia P2P node and the gRPC server concurrently.
pub async fn run_concurrent_services() -> Result<(), Box<dyn Error>> {
    // Create communication channel
    let (sender, receiver) = mpsc::unbounded_channel();

    // Crete instances
    let grpc_daemon = GrpcDaemon::new(sender);
    let p2p_kad = P2pKad::new(receiver);

    let grpc_handle: JoinHandle<()> = tokio::spawn(async move {
        let addr = "[::1]:50051".parse().unwrap();
        println!("Starting (gRPC Server) on {}...", addr);
        let _ = grpc_daemon.run_server(addr).await;
    });

    let p2p_handle = tokio::spawn(async move {
        println!("Starting (P2P Kademlia)...");
        let _ = p2p_kad.run().await;
    });

    tokio::try_join!(grpc_handle, p2p_handle)?;

    Ok(())
}
