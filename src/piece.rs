use sha3::{Digest, Sha3_512};
use digest::FixedOutput;
use generic_array::GenericArray;

use std::fmt::{Debug, Formatter};
use std::fmt;

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub struct PieceRef {
    sha3_512: GenericArray<u8, <Sha3_512 as FixedOutput>::OutputSize>,
}

impl Debug for PieceRef {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:X}", self.sha3_512)
    }
}

impl PieceRef {
    pub fn for_buffer(buf: &[u8]) -> PieceRef {
        PieceRef {
            sha3_512: Sha3_512::digest(buf),
        }
    }
}
