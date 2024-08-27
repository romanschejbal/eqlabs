// pub use super::{Decode, Encode, Error, Message, Result};
use super::{decode::Decode, encode::Encode, error::Error, protocol::Message};
use bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder};

pub struct BitcoinCodec;

impl Encoder<Message> for BitcoinCodec {
    type Error = Error;

    fn encode(&mut self, message: Message, dst: &mut BytesMut) -> Result<(), Self::Error> {
        message.encode(dst);
        Ok(())
    }
}

impl Decoder for BitcoinCodec {
    type Item = Message;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() || src.len() < 24 {
            // Not enough bytes
            return Ok(None);
        }

        // Length of payload starts at 16th byte, and is 4 bytes long
        let payload_length = u32::from_le_bytes([src[16], src[17], src[18], src[19]]);

        if src.len() < 24 + payload_length as usize {
            // Not enough bytes
            return Ok(None);
        }

        let message = Message::decode(src)?;

        Ok(Some(message))
    }
}
