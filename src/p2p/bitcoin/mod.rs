//! # Bitcoin protocol handshake
//! Implementation based on [Protocol documentation](https://en.bitcoin.it/wiki/Protocol_documentation) on Wikipedia.

mod codec;
mod decode;
mod encode;
mod error;
mod handshake;
mod hashes;
mod protocol;

use decode::Decode;
use encode::Encode;
use error::{Error, Result};

pub use handshake::*;
