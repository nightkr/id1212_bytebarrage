use blake2::Blake2b;
use digest::{Digest, FixedOutput};
use generic_array::GenericArray;
use serde::{de, Deserializer};
use serde_bytes;
use typenum::Unsigned;

use std::fmt::{self, Debug, Formatter};
use std::io::{self, Read};

type Length = <Blake2b as FixedOutput>::OutputSize;

fn deserialize_hash_arr<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<GenericArray<u8, Length>, D::Error> {
    let buf = serde_bytes::deserialize::<Vec<u8>, D>(deserializer)?;
    if buf.len() == Length::to_usize() {
        Ok(GenericArray::clone_from_slice(&buf))
    } else {
        let msg = format!("a byte array of length {}", Length::to_usize());
        Err(de::Error::invalid_length(buf.len(), &msg.as_str()))
    }
}

#[derive(Hash, Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct PieceRef {
    #[serde(serialize_with = "serde_bytes::serialize", deserialize_with = "deserialize_hash_arr")]
    blake2b_64: GenericArray<u8, Length>,
}

impl Debug for PieceRef {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:X}", self.blake2b_64)
    }
}

impl PieceRef {
    pub fn for_buffer(buf: &[u8]) -> PieceRef {
        PieceRef {
            blake2b_64: Blake2b::digest(buf),
        }
    }

    pub fn verify(&self, buf: &[u8]) -> bool {
        *self == Self::for_buffer(buf)
    }

    pub fn to_vec(&self) -> Vec<u8> {
        Vec::from(self.blake2b_64.as_slice())
    }

    pub fn from_read<R: Read>(read: &mut R) -> io::Result<PieceRef> {
        let mut hash_buf = GenericArray::default();
        read.read_exact(hash_buf.as_mut_slice())?;
        Ok(PieceRef {
            blake2b_64: hash_buf,
        })
    }
}
