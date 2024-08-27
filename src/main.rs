mod p2p;

use p2p::bitcoin;

/// For Bitcoin handshake
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let endpoint = "seed.bitcoin.sipa.be:8333";
    tracing::info!("Connecting to {endpoint}");
    let handshake = bitcoin::Handshake::connect(endpoint).await?;

    Ok(())
}

// For ethereum handshake
// #[tokio::main]
// async fn main() -> anyhow::Result<()> {
//     // https://ethernodes.org/node/00022472a33bf4be92599db8d2a284599141dcbeea0610f88887e631e5531d90c926aeb1ca003dc4d99ecb1e43c3472d4d2006ebb0c38f51d7b7470c91f767b5#
//     let ethereum_enode = "enode://00022472a33bf4be92599db8d2a284599141dcbeea0610f88887e631e5531d90c926aeb1ca003dc4d99ecb1e43c3472d4d2006ebb0c38f51d7b7470c91f767b5@82.66.183.172:30303";

//     let (id, addr) = ethereum_enode.split_once("@").expect("Infallible");
//     let node_pubk = id.split("://").last().expect("Infallible");
//     let tcp_stream = TcpStream::connect(addr).await?;

//     println!("Public key: {node_pubk} {}", node_pubk.len());
//     let node_pubk_bytes = hex::decode(node_pubk)?;

//     // Ensure the public key is in uncompressed format (64 bytes)
//     assert_eq!(
//         node_pubk_bytes.len(),
//         64,
//         "Public key must be 64 bytes long"
//     );

//     // Add the 0x04 prefix to denote an uncompressed public key
//     let mut uncompressed_public_key = vec![0x04];
//     uncompressed_public_key.extend_from_slice(&node_pubk_bytes);

//     let node_pubk = PublicKey::from_slice(&uncompressed_public_key)
//         .context("could not parse node public key")?;

//     let mut framed = Framed::new(tcp_stream, RLPx::new(node_pubk));

//     // let message = framed.next().await;

//     println!("Sending auth");
//     framed.send(Message::Auth).await?;
//     println!("Sent auth");

//     println!("Receiving auth");

//     loop {
//         let message = framed.next().await;
//         tokio::task::yield_now().await;
//     }
// }
