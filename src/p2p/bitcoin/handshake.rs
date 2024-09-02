use super::{
    codec::BitcoinCodec,
    protocol::{Address, Command, Message, Payload, VersionMessage},
};
use anyhow::Result;
use futures::{SinkExt, StreamExt};
use tokio::{
    net::{TcpStream, ToSocketAddrs},
    sync::mpsc::{Receiver, Sender},
};
use tokio_util::codec::Framed;

pub struct Handshake {
    stream_rx: Receiver<Message>,
    sink_tx: Sender<Message>,
}

impl Handshake {
    pub async fn connect(address: impl ToSocketAddrs) -> Result<Self> {
        let stream = TcpStream::connect(address).await?;
        tracing::debug!("Connection established");

        let framed_stream = Framed::new(stream, BitcoinCodec);
        let (mut sink, mut stream) = framed_stream.split();
        let (sink_tx, mut sink_rx) = tokio::sync::mpsc::channel(1);
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
        let (stream_tx, stream_rx) = tokio::sync::mpsc::channel(1);

        tokio::spawn(async move {
            while let Some(message) = sink_rx.recv().await {
                if let Err(e) = sink.send(message).await {
                    tracing::error!("Error: {}", e);
                    break;
                }
            }
        });

        let sink_tx_inner = sink_tx.clone();
        tokio::spawn(async move {
            let version_message = VersionMessage {
                version: 70016,
                timestamp: 0,
                user_agent: "/ramen/".into(),
                addr_recv: Address {
                    time: (),
                    services: 0,
                    ip: "::".parse().unwrap(),
                    port: 0.into(),
                },
                addr_from: Address {
                    time: (),
                    services: 0,
                    ip: "::".parse().unwrap(),
                    port: 0.into(),
                },
                nonce: 0,
                services: 0,
                start_height: 0,
                relay: false,
            };
            let message = Message::new(
                0xD9B4BEF9,
                Command::Version,
                Payload::Version(version_message),
            );
            tracing::info!("Sending version message: {message:?}");
            let _ = sink_tx_inner.send(message).await;

            while let Some(message) = stream.next().await {
                let message = match message {
                    Ok(message) => message,
                    Err(e) => {
                        tracing::error!("Error: {}", e);
                        continue;
                    }
                };

                match message.payload() {
                    Payload::Version(version) => {
                        tracing::info!("Version message received: {:?}", version);
                        let message = Message::new(0xD9B4BEF9, Command::VerAck, Payload::VerAck);
                        tracing::info!("Sending verack: {:?}", message);
                    }
                    Payload::VerAck => {
                        tracing::info!("Verack message received");
                        let _ = ready_tx.send(());
                        break;
                    }
                    Payload::SendHeaders => {
                        tracing::info!("SendHeaders received");
                    }
                    Payload::Empty => {
                        tracing::info!("Empty payload received");
                    }
                }
            }

            while let Some(message) = stream.next().await {
                match message {
                    Ok(message) => {
                        if let Err(e) = stream_tx.send(message).await {
                            tracing::error!("Error: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        tracing::error!("Error: {}", e);
                        continue;
                    }
                }
            }
        });

        ready_rx.await?;

        Ok(Self { stream_rx, sink_tx })
    }

    pub fn split(self) -> (Sender<Message>, Receiver<Message>) {
        (self.sink_tx, self.stream_rx)
    }
}
