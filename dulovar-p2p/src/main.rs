use dulovar_p2p::{config, orchestrator::run_concurrent_services};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let _app_config = match config::load_config() {
        Ok(c) => run_concurrent_services(c).await,
        Err(e) => {
            eprintln!("Fatal Error: Could not load configuration.");
            // Re-throw the error to exit the program gracefully
            return Err(e);
        }
    };

    Ok(())
}
