use anyhow::Result;
use iroh::{Endpoint, SecretKey};

pub struct P2pNode {
    active: bool,
}

impl P2pNode {
    
    #[tokio::main]
    pub async fn generate_end_point() ->  Result<()>  {

        let secret_key = SecretKey::generate(futures_rand::rngs::OsRng);

        // Create an endpoint.
        let endpoint = Endpoint::builder()
            .secret_key(secret_key)
            .discovery_n0()
            .bind()
            .await?;

        println!("> our node id: {}", endpoint.node_id());

        Ok(())
    }
}
