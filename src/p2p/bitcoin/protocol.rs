use super::{hashes::Checksum, Decode, Encode, Error, Result};
use bytes::{Buf, BufMut, BytesMut};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Message {
    magic: u32,
    command: Command,
    length: u32,
    checksum: u32,
    payload: Payload,
}

impl Message {
    pub fn new(magic: u32, command: Command, payload: Payload) -> Self {
        let mut payload_encoded = BytesMut::new();
        let length = payload.encode(&mut payload_encoded) as u32;
        let checksum = payload_encoded.sha256();
        Self {
            magic,
            command,
            length,
            checksum,
            payload,
        }
    }

    pub fn payload(&self) -> &Payload {
        &self.payload
    }
}

impl Encode for Message {
    fn encode(&self, buffer: &mut BytesMut) -> usize {
        let mut written = self.magic.encode(buffer);
        written += self.command.encode(buffer);
        written += self.length.encode(buffer);
        written += self.checksum.encode(buffer);
        written += self.payload.encode(buffer);
        written
    }
}

impl Decode for Message {
    fn decode(bytes: &mut BytesMut) -> Result<Self> {
        let magic = u32::decode(bytes)?;
        let command = Command::decode(bytes)?;
        let length = u32::decode(bytes)?;
        let checksum = u32::decode(bytes)?;
        let payload = Payload::decode_command(&command, bytes)?;
        Ok(Message {
            magic,
            command,
            length,
            checksum,
            payload,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Command {
    Version,
    VerAck,
    SendHeaders,
    WtxIdRelay,
    SendAddrV2,
}

impl Command {
    fn decode(bytes: &mut BytesMut) -> Result<Self> {
        let decode = match &bytes[..12] {
            b"version\0\0\0\0\0" => Ok(Command::Version),
            b"verack\0\0\0\0\0\0" => Ok(Command::VerAck),
            b"wtxidrelay\0\0" => Ok(Command::WtxIdRelay),
            b"sendheaders\0" => Ok(Command::SendHeaders),
            b"sendaddrv2\0\0" => Ok(Command::SendAddrV2),
            x => Err(Error::Command(format!(
                "unhandled command: {:?}",
                String::from_utf8_lossy(x)
            ))),
        };
        bytes.advance(12);
        decode
    }
}

impl Encode for Command {
    fn encode(&self, buffer: &mut BytesMut) -> usize {
        match self {
            Self::Version => buffer.put_slice(b"version\0\0\0\0\0"),
            Self::VerAck => buffer.put_slice(b"verack\0\0\0\0\0\0"),
            Self::WtxIdRelay => buffer.put_slice(b"wtxidrelay\0\0\0\0\0\0"),
            Self::SendHeaders => buffer.put_slice(b"sendheaders\0"),
            Self::SendAddrV2 => buffer.put_slice(b"sendaddrv2\0\0"),
        };
        12
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Payload {
    Version(VersionMessage),
    VerAck,
    SendHeaders,
    Empty,
}

impl Payload {
    fn decode_command(command: &Command, bytes: &mut BytesMut) -> Result<Self> {
        match command {
            Command::Version => {
                let version = VersionMessage::decode(bytes)?;
                Ok(Payload::Version(version))
            }
            Command::VerAck => Ok(Payload::VerAck),
            Command::WtxIdRelay => Ok(Payload::Empty),
            Command::SendHeaders => Ok(Payload::SendHeaders),
            Command::SendAddrV2 => Ok(Payload::Empty),
        }
    }
}

impl Encode for Payload {
    fn encode(&self, buffer: &mut BytesMut) -> usize {
        match self {
            Self::Version(version) => version.encode(buffer),
            Self::VerAck => ().encode(buffer),
            Self::SendHeaders => ().encode(buffer),
            Self::Empty => ().encode(buffer),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VersionMessage {
    pub version: i32,
    pub services: u64,
    pub timestamp: i64,
    pub addr_recv: Address<()>,
    pub addr_from: Address<()>,
    pub nonce: u64,
    pub user_agent: VariableLengthString,
    pub start_height: i32,
    pub relay: bool,
}

impl Encode for VersionMessage {
    fn encode(&self, buffer: &mut BytesMut) -> usize {
        let mut written = self.version.encode(buffer);
        written += self.services.encode(buffer);
        written += self.timestamp.encode(buffer);
        written += self.addr_recv.encode(buffer);
        written += self.addr_from.encode(buffer);
        written += self.nonce.encode(buffer);
        written += self.user_agent.encode(buffer);
        written += self.start_height.encode(buffer);
        written += self.relay.encode(buffer);
        written
    }
}

impl Decode for VersionMessage {
    fn decode(bytes: &mut BytesMut) -> Result<Self> {
        let version = i32::decode(bytes)?;
        let services = u64::decode(bytes)?;
        let timestamp = i64::decode(bytes)?;
        let addr_recv = Address::decode(bytes)?;
        let addr_from = Address::decode(bytes)?;
        let nonce = u64::decode(bytes)?;
        let user_agent = VariableLengthString::decode(bytes)?;
        let start_height = i32::decode(bytes)?;
        let relay = bool::decode(bytes)?;
        Ok(VersionMessage {
            version,
            services,
            timestamp,
            addr_recv,
            addr_from,
            nonce,
            user_agent,
            start_height,
            relay,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Address<T> {
    pub time: T,
    pub services: u64,
    pub ip: std::net::IpAddr,
    pub port: Port,
}

impl<T> Encode for Address<T>
where
    T: Encode,
{
    fn encode(&self, buffer: &mut BytesMut) -> usize {
        let mut bytes = 0;
        bytes += self.time.encode(buffer);
        bytes += self.services.encode(buffer);
        bytes += self.ip.encode(buffer);
        bytes += self.port.encode(buffer);
        bytes
    }
}

impl<T> Decode for Address<T>
where
    T: Decode,
{
    fn decode(bytes: &mut BytesMut) -> Result<Self> {
        let time = T::decode(bytes)?;
        let services = u64::decode(bytes)?;
        let ip = std::net::IpAddr::decode(bytes)?;
        let port = Port::decode(bytes)?;
        Ok(Address {
            time,
            services,
            ip,
            port,
        })
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Port(u16);

impl From<u16> for Port {
    fn from(port: u16) -> Self {
        Self(port)
    }
}

impl Encode for Port {
    fn encode(&self, buffer: &mut BytesMut) -> usize {
        buffer.put_u16(self.0);
        std::mem::size_of::<u16>()
    }
}

impl Decode for Port {
    fn decode(bytes: &mut BytesMut) -> Result<Self> {
        if bytes.remaining() <= 2 {
            return Err(Error::NotEnoughBytes("port"));
        };
        Ok(Self(bytes.get_u16()))
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
struct VariableInt(u64);

impl Encode for VariableInt {
    fn encode(&self, buffer: &mut BytesMut) -> usize {
        match self.0 {
            0..=0xFC => {
                buffer.put_u8(self.0 as u8);
                1
            }
            0xFD..=0xFFFF => {
                buffer.put_u8(0xFD);
                1 + (self.0 as u16).encode(buffer)
            }
            0x10000..=0xFFFFFFFF => {
                buffer.put_u8(0xFE);
                1 + (self.0 as u32).encode(buffer)
            }
            _ => {
                buffer.put_u8(0xFE);
                self.0.encode(buffer);
                1 + self.0.encode(buffer)
            }
        }
    }
}

impl Decode for VariableInt {
    fn decode(bytes: &mut BytesMut) -> Result<Self> {
        if bytes.remaining() < 1 {
            return Err(Error::NotEnoughBytes("variable int"));
        }
        match bytes.get_u8() {
            0xFD => {
                let number = u16::decode(bytes)?;
                Ok(VariableInt(number as u64))
            }
            0xFE => {
                let number = u32::decode(bytes)?;
                Ok(VariableInt(number as u64))
            }
            0xFF => {
                let number = u64::decode(bytes)?;
                Ok(VariableInt(number))
            }
            x => Ok(VariableInt(x as u64)),
        }
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct VariableLengthString(VariableInt, String);

impl From<&str> for VariableLengthString {
    fn from(s: &str) -> Self {
        VariableLengthString(VariableInt(s.len() as u64), s.to_string())
    }
}

impl Encode for VariableLengthString {
    fn encode(&self, buffer: &mut BytesMut) -> usize {
        let written = self.0.encode(buffer);
        buffer.put_slice(self.1.as_bytes());
        written + self.1.len()
    }
}

impl Decode for VariableLengthString {
    fn decode(bytes: &mut BytesMut) -> Result<Self> {
        let length = VariableInt::decode(bytes)?;
        if bytes.remaining() < length.0 as usize {
            return Err(Error::NotEnoughBytes("variable length string"));
        }
        let str = String::from_utf8_lossy(&bytes[..length.0 as usize]).into_owned();
        bytes.advance(length.0 as usize);
        Ok(Self(length, str))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn encode() {
        let message_bin = b"\xf9\xbe\xb4\xd9version\0\0\0\0\0f\0\0\0@e\xe2A\x80\x11\x01\0\t\x04\0\0\0\0\0\0\x0e\xb1$d\0\0\0\0\0\0\0\0\0\0\0\0*\x02\x83\x08\x90\x0cY\0\xb5\x9b\xb5Q\x1c&\x02\xa8\xdb~\t\x04\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0SH\x1f\xe5\xdc6S`\x10/Satoshi:23.0.0/\xe8\xf2\x0b\0\x01";
        let message = Message {
            magic: 3652501241,
            command: Command::Version,
            length: 102,
            checksum: 1105356096,
            payload: Payload::Version(VersionMessage {
                version: 70016,
                services: 1033,
                timestamp: 1680126222,
                addr_recv: Address {
                    time: (),
                    services: 0,
                    ip: "2a02:8308:900c:5900:b59b:b551:1c26:2a8".parse().unwrap(),
                    port: Port(56190),
                },
                addr_from: Address {
                    time: (),
                    services: 1033,
                    ip: "::".parse().unwrap(),
                    port: Port(0),
                },
                nonce: 6940951773072803923,
                user_agent: "/Satoshi:23.0.0/".into(),
                start_height: 783080,
                relay: true,
            }),
        };

        let mut buffer = BytesMut::new();
        message.encode(&mut buffer);
        assert_eq!(&buffer[..], message_bin);
    }

    #[test]
    fn decode() {
        let message_bin = b"\xf9\xbe\xb4\xd9version\0\0\0\0\0f\0\0\0@e\xe2A\x80\x11\x01\0\t\x04\0\0\0\0\0\0\x0e\xb1$d\0\0\0\0\0\0\0\0\0\0\0\0*\x02\x83\x08\x90\x0cY\0\xb5\x9b\xb5Q\x1c&\x02\xa8\xdb~\t\x04\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0SH\x1f\xe5\xdc6S`\x10/Satoshi:23.0.0/\xe8\xf2\x0b\0\x01\xf9\xbe\xb4\xd9verack\0\0\0\0\0\0\0\0\0\0]\xf6\xe0\xe2";
        let mut bytes = BytesMut::new();
        bytes.extend_from_slice(&message_bin[..]);
        let message = Message::decode(&mut bytes).unwrap();
        assert_eq!(
            message,
            Message {
                magic: 3652501241,
                command: Command::Version,
                length: 102,
                checksum: 1105356096,
                payload: Payload::Version(VersionMessage {
                    version: 70016,
                    services: 1033,
                    timestamp: 1680126222,
                    addr_recv: Address {
                        time: (),
                        services: 0,
                        ip: "2a02:8308:900c:5900:b59b:b551:1c26:2a8".parse().unwrap(),
                        port: Port(56190),
                    },
                    addr_from: Address {
                        time: (),
                        services: 1033,
                        ip: "::".parse().unwrap(),
                        port: Port(0),
                    },
                    nonce: 6940951773072803923,
                    user_agent: "/Satoshi:23.0.0/".into(),
                    start_height: 783080,
                    relay: true,
                })
            }
        );
    }

    #[test]
    fn encode_decode() {
        let msg = Message {
            magic: 3652501241,
            command: Command::Version,
            length: 102,
            checksum: 1105356096,
            payload: Payload::Version(VersionMessage {
                version: 70016,
                services: 1033,
                timestamp: 1680126222,
                addr_recv: Address {
                    time: (),
                    services: 0,
                    ip: "2a02:8308:900c:5900:b59b:b551:1c26:2a8".parse().unwrap(),
                    port: Port(56190),
                },
                addr_from: Address {
                    time: (),
                    services: 1033,
                    ip: "::192.168.0.1".parse().unwrap(),
                    port: Port(0),
                },
                nonce: 6920951773072803923,
                user_agent: "/Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto/".into(),
                start_height: 1932515342,
                relay: true,
            })
        };

        let mut buf = BytesMut::new();
        let _ = msg.encode(&mut buf);
        let decoded = Message::decode(&mut buf).unwrap();

        assert_eq!(decoded, msg);
    }

    #[test]
    fn test_payload_length() {
        let payload = Payload::Version(VersionMessage {
            version: 70016,
            services: 1033,
            timestamp: 1680126222,
            addr_recv: Address {
                time: (),
                services: 0,
                ip: "2a02:8308:900c:5900:b59b:b551:1c26:2a8".parse().unwrap(),
                port: Port(56190),
            },
            addr_from: Address {
                time: (),
                services: 1033,
                ip: "::192.168.0.1".parse().unwrap(),
                port: Port(0),
            },
            nonce: 6920951773072803923,
            user_agent: "/Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto//Satoshi:23.0.0:Nakamoto/".into(),
            start_height: 1932515342,
            relay: true,
        });
        assert_eq!(payload.encode(&mut BytesMut::new()), 813);
    }
}
