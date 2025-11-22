use std::collections::HashMap;

use reqwest::Client;
use serde::Deserialize;

use crate::config::Configuration;

#[derive(Deserialize)]
struct Node {
    address: String,
}

pub struct RestRequest {
    pub c: Configuration,
}

impl RestRequest {
    pub fn new(c: Configuration) -> RestRequest {
        RestRequest { c }
    }

    pub async fn get_nodes(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let url = self.c.node_web_server_url.clone();
        let response = reqwest::get(url).await?;
        let nodes: Vec<Node> = response.json().await?;
        Ok(nodes.into_iter().map(|n| n.address).collect())
    }

    pub async fn register_node(&self) -> Result<(), reqwest::Error> {
        let ip = self.get_public_ip().await;
        let mut map = HashMap::new();
        map.insert("address", &ip);

        let url = self.c.node_web_server_url.clone();
        let client = Client::new();

        let res = client.post(url).json(&map).send().await?;

        if res.status().is_success() {
            println!("Node registered successfully with IP: {}", ip);
        } else {
            eprintln!("Error to register node. Code: {}", res.status());
        }

        Ok(())
    }

    pub async fn get_public_ip(&self) -> String {
        match reqwest::get(self.c.public_ip_url_1.clone()).await {
            Ok(resp) if resp.status().is_success() => match resp.text().await {
                Ok(t) => t.trim().to_string(),
                Err(e) => {
                    eprintln!("Failed to read ip body from api.ipify.org: {e}");
                    String::from("unknown")
                }
            },
            _ => match reqwest::get(self.c.public_ip_url_1.clone()).await {
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
        }
    }
}
