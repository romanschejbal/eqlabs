use hmac::{Hmac, Mac};
use rand::RngCore;
use rlp::RlpStream;
use secp256k1::{PublicKey, Secp256k1, SecretKey};
use sha2::Sha256;
use sha3::{Digest, Keccak256};
use tokio_util::codec::{Decoder, Encoder};

use super::message::Message;

struct RLPx {
    ephemeral_pubk: PublicKey,
    ephemeral_seck: SecretKey,
    initiator_pubk: PublicKey,
    initiator_seck: SecretKey,
    receiver_pubk: PublicKey,
}

impl RLPx {
    pub fn new(receiver_pubk: PublicKey) -> Self {
        let secp = Secp256k1::new();
        let (ephemeral_seck, ephemeral_pubk) = secp.generate_keypair(&mut rand::thread_rng());
        let (initiator_seck, initiator_pubk) = secp.generate_keypair(&mut rand::thread_rng());

        Self {
            ephemeral_seck,
            ephemeral_pubk,
            initiator_pubk,
            initiator_seck,
            receiver_pubk,
        }
    }
}

impl Encoder<Message> for RLPx {
    type Error = anyhow::Error;

    fn encode(&mut self, item: Message, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        match item {
            Message::Auth => {
                let mut auth_body = RlpStream::new_list(4);

                // Create a new secp256k1 context and generate an ephemeral keypair

                // Hash the ephemeral public key
                let ephemeral_pubk_hash = {
                    let mut hasher = Keccak256::new();
                    hasher.update(self.ephemeral_pubk.serialize_uncompressed());
                    hasher.finalize()
                };

                // Sign the hash using the initiator's static private key
                let secp = Secp256k1::new();
                let signature = secp.sign_ecdsa(
                    &secp256k1::Message::from_digest_slice(&ephemeral_pubk_hash[..])?,
                    &self.initiator_seck,
                );

                // Append the signature to the auth body
                auth_body.append(&&signature.serialize_compact()[..]);

                // Append the initiator's ephemeral public key
                auth_body.append(&&self.ephemeral_pubk.serialize()[..]);

                // Generate and append a random nonce
                let mut iv = [0_u8; 32];
                rand::rngs::OsRng.try_fill_bytes(&mut iv)?;
                auth_body.append(&&iv[..]);

                // Append the auth version
                auth_body.append(&5_u8);

                // Convert the RLP-encoded auth body to bytes
                let auth_body = auth_body.out();

                // Compute the shared secret
                let shared_secret =
                    secp256k1::ecdh::SharedSecret::new(&self.receiver_pubk, &self.ephemeral_seck)
                        .secret_bytes();

                // Derive symmetric encryption and MAC keys from the shared secret
                let key = concat_kdf::derive_key::<sha3::Sha3_256>(&shared_secret, &[], 32)?;

                let enc_key = &key[0..16];
                let mac_key = &key[16..32];

                // Encrypt the message using the derived key
                let cipher = openssl::symm::Cipher::aes_128_ctr();
                let iv = [0_u8; 16];
                let c = openssl::symm::encrypt(cipher, enc_key, Some(&iv), &auth_body)?;

                // Generate a tag using HMAC-SHA256
                type HmacSha256 = Hmac<Sha256>;
                let mut d = HmacSha256::new_from_slice(mac_key)?;
                d.update(&[&iv[..], &c[..]].concat());
                let d = d.finalize().into_bytes();

                // Alice sends the encrypted message R || iv || c || d where c = AES(kE, iv , m) and d = MAC(sha256(kM), iv || c) to Bob.
                let messsage = [
                    &self.initiator_pubk.serialize_uncompressed()[..],
                    &iv[..],
                    &c[..],
                    &d[..],
                ]
                .concat();

                // Calculate auth-size and prepend it to dst
                let size = messsage.len() as i16;

                dst.extend_from_slice(&size.to_be_bytes());

                // Append the encrypted auth body to dst
                dst.extend_from_slice(&c[..]);
            }
        }
        Ok(())
    }
}

impl Decoder for RLPx {
    type Item = Message;
    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let len = src.len();
        let message = src.split_to(len);

        Ok(None)
    }
}
