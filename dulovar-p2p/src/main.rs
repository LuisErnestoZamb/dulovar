mod p2p_kad;
use p2p_kad::*;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = init_kad::init_kad().await?;
    Ok(())
}
