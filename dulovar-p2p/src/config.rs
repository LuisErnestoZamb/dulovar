use serde::Deserialize;
use std::fs;
use std::io::{self, ErrorKind};

// Define the struct (must be public to be used in other files)
#[derive(Debug, Deserialize, Clone)]
pub struct Configuration {
    pub grpc_port: u16,
    pub p2p_port: u16,
    pub production: bool,
    pub private_key_file: String,
    pub node_web_server_url: String,
    pub public_ip_url_1: String,
    pub public_ip_url_2: String,
}

/// Loads the configuration from a YAML file at the specified path.
pub fn load_config() -> Result<Configuration, io::Error> {
    let file_path = "config.yaml";

    // 1. Read the YAML file content into a String.
    println!("Attempting to read config from: {}", file_path);
    let file_content = fs::read_to_string(file_path)?;

    // 2. Use serde_yaml to deserialize the String into the Configuration struct.
    match serde_yaml::from_str::<Configuration>(&file_content) {
        Ok(c) => {
            println!("Configuration successfully parsed.");
            Ok(c)
        }
        Err(e) => {
            // Convert the serde_yaml error into a standard IO error for consistent handling
            Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("YAML parsing failed: {}", e),
            ))
        }
    }
}
