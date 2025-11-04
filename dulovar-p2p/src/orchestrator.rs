use std::error::Error;
use tokio;

use crate::grpc_daemon::alert_service::run_server;
use crate::p2p_kad::init_kad;

/// Runs the Kademlia P2P node and the gRPC server concurrently.
pub async fn run_concurrent_services() -> Result<(), Box<dyn Error>> {
    // ---  Setup (P2P Kademlia) ---
    let task_a = async {
        println!("Starting (P2P Kademlia)...");
        init_kad::init_kad().await
    };

    // ---  Setup (gRPC Server) ---
    let task_b = async {
        let addr = "[::1]:50051".parse().unwrap();
        println!("Starting (gRPC Server) on {}...", addr);
        run_server(addr).await
    };

    // The tokio::join! macro runs both tasks in parallel and waits for them to finish.
    let (res_a, res_b) = tokio::join!(task_a, task_b);

    // --- Error Handling ---

    // Process  (Kademlia) result
    match res_a {
        Ok(_) => println!(" (Kademlia) finished successfully."),
        Err(ref e) => eprintln!("Fatal error in  (Kademlia): {}", e),
    }

    // Process  (gRPC) result
    match res_b {
        Ok(_) => println!(" (gRPC) finished successfully."),
        Err(ref e) => eprintln!("Fatal error in  (gRPC): {}", e),
    }

    // Return an error if any task failed.
    res_a.and(res_b)
}
