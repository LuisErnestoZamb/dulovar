mod p2p_kad;
use p2p_kad::P2pKad;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = P2pKad::init_kad().await?;
    Ok(())
}
