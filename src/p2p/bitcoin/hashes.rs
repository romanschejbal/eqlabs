use bytes::BytesMut;

pub trait Checksum {
    fn sha256(&self) -> u32;
}

impl Checksum for BytesMut {
    fn sha256(&self) -> u32 {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(self);
        let result = hasher.finalize();
        let mut hasher = Sha256::new();
        hasher.update(result);
        let result = hasher.finalize();
        let mut bytes = [0; 4];
        bytes.copy_from_slice(&result[0..4]);
        u32::from_le_bytes(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkusum_test() {
        let data: [u8; 4] = [0x01, 0x02, 0x03, 0x04];
        let data = BytesMut::from_iter(data.iter());
        assert_eq!(data.sha256(), 3799180429);
    }
}
