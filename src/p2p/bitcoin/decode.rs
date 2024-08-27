use super::error::Error;
use bytes::{Buf, BytesMut};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

type Result<T> = std::result::Result<T, Error>;

pub trait Decode
where
    Self: Sized,
{
    fn decode(buffer: &mut BytesMut) -> Result<Self>;
}

impl Decode for () {
    fn decode(_buffer: &mut BytesMut) -> Result<Self> {
        Ok(())
    }
}

impl Decode for bool {
    fn decode(buffer: &mut BytesMut) -> Result<Self> {
        if buffer.remaining() < std::mem::size_of::<bool>() {
            return Err(Error::NotEnoughBytes("bool"));
        }
        Ok(buffer.get_u8() != 0)
    }
}

macro_rules! make_decoder {
    ($t: ty, $fn: ident) => {
        impl Decode for $t {
            fn decode(buffer: &mut BytesMut) -> Result<Self> {
                let len = std::mem::size_of::<$t>();
                if buffer.len() < len {
                    return Err(Error::NotEnoughBytes(stringify!($t)));
                }
                let value = buffer.$fn();
                println!("Decoded {}: {}", stringify!($t), value);
                Ok(value)
            }
        }
    };
}

make_decoder!(u8, get_u8);
make_decoder!(u16, get_u16_le);
make_decoder!(u32, get_u32_le);
make_decoder!(u64, get_u64_le);
make_decoder!(i8, get_i8);
make_decoder!(i16, get_i16_le);
make_decoder!(i32, get_i32_le);
make_decoder!(i64, get_i64_le);

impl Decode for IpAddr {
    fn decode(buffer: &mut BytesMut) -> Result<Self> {
        if buffer.remaining() < std::mem::size_of::<IpAddr>() {
            return Err(Error::NotEnoughBytes("IpAddr"));
        }
        let ip = TryInto::<[u8; 16]>::try_into(&buffer[..16])?.into();
        buffer.advance(16);
        return Ok(ip);
    }
}
