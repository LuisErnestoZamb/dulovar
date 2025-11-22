use libp2p::identity;

pub fn read_keypair_file(
    file_name: String,
) -> Result<identity::Keypair, Box<dyn std::error::Error>> {
    let id_keys = if std::path::Path::new(&file_name).exists() {
        let key_bytes = std::fs::read(file_name)?;
        identity::Keypair::from_protobuf_encoding(&key_bytes)?
    } else {
        let id_keys = identity::Keypair::generate_ed25519();
        std::fs::write(file_name, id_keys.to_protobuf_encoding()?)?;
        id_keys
    };

    Ok(id_keys)
}
