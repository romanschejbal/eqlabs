use bytes::{BufMut, BytesMut};
use std::net::IpAddr;

pub trait Encode {
    fn encode(&self, buffer: &mut BytesMut) -> usize;
}

impl Encode for bool {
    fn encode(&self, buffer: &mut BytesMut) -> usize {
        buffer.put_u8(*self as u8);
        std::mem::size_of::<bool>()
    }
}

impl Encode for () {
    fn encode(&self, _buffer: &mut BytesMut) -> usize {
        0
    }
}

macro_rules! make_encoder {
    ($t: ty, $fn: ident) => {
        impl Encode for $t {
            fn encode(&self, buffer: &mut BytesMut) -> usize {
                buffer.$fn(*self);
                std::mem::size_of::<$t>()
            }
        }
    };
}

make_encoder!(u8, put_u8);
make_encoder!(u16, put_u16_le);
make_encoder!(u32, put_u32_le);
make_encoder!(u64, put_u64_le);
make_encoder!(i8, put_i8);
make_encoder!(i16, put_i16_le);
make_encoder!(i32, put_i32_le);
make_encoder!(i64, put_i64_le);

impl Encode for IpAddr {
    fn encode(&self, buffer: &mut BytesMut) -> usize {
        match self {
            IpAddr::V4(ip) => buffer.put_slice(&[[0; 4], [0; 4], [0; 4], ip.octets()].concat()),
            IpAddr::V6(ip) => buffer.put_slice(&ip.octets()),
        }
        std::mem::size_of::<IpAddr>()
    }
}
