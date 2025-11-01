use std::collections::HashMap;

use reqwest::Client;
use serde::Deserialize;

pub struct RestRequest;

#[derive(Deserialize)]
struct Node {
    address: String,
    valid: i32,
    master: i32,
}

impl RestRequest {
    const DOMAIN: &str = "https://api.dulovar.com/nodes";

    pub async fn get_nodes() -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let url = Self::DOMAIN;
        let response = reqwest::get(url).await?;
        let nodes: Vec<Node> = response.json().await?;
        Ok(nodes.into_iter().map(|n| n.address).collect())
    }

    pub async fn register_node() -> Result<(), reqwest::Error> {
        let ip = Self::get_public_ip().await;
        let mut map = HashMap::new();
        map.insert("address", &ip);

        let url = Self::DOMAIN;
        let client = Client::new();

        let res = client.post(url).json(&map).send().await?;

        if res.status().is_success() {
            println!("Node registered successfully with IP: {}", ip);
        } else {
            eprintln!("Error to register node. Code: {}", res.status());
        }

        Ok(())
    }

    pub async fn get_public_ip() -> String {
        let public_ip = match reqwest::get("https://api.ipify.org?format=text").await {
            Ok(resp) if resp.status().is_success() => match resp.text().await {
                Ok(t) => t.trim().to_string(),
                Err(e) => {
                    eprintln!("Failed to read ip body from api.ipify.org: {e}");
                    String::from("unknown")
                }
            },
            _ => match reqwest::get("https://ifconfig.co/ip").await {
                Ok(resp) if resp.status().is_success() => {
                    resp.text().await.unwrap_or_default().trim().to_string()
                }
                Ok(resp) => {
                    eprintln!(
                        "Fallback service returned non-success status: {}",
                        resp.status()
                    );
                    String::from("unknown")
                }
                Err(e) => {
                    eprintln!("Failed to contact public IP services: {e}");
                    String::from("unknown")
                }
            },
        };
        public_ip
    }
}
