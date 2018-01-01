use super::piece::PieceRef;

use std::error;
use std::fmt::{self, Display, Formatter};
use std::path::Path;
use std::fs::File;
use std::io::{self, Read};

use rmp_serde;

const PIECE_BYTES: usize = 1024 * 1024;

#[derive(Debug)]
pub enum Error {
    EncodeFailed(rmp_serde::encode::Error),
    DecodeFailed(rmp_serde::decode::Error),
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::EncodeFailed(_) => "encode failed",
            Error::DecodeFailed(_) => "decode failed",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::EncodeFailed(ref err) => Some(err),
            Error::DecodeFailed(ref err) => Some(err),
        }
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        error::Error::description(self).fmt(formatter)
    }
}

impl From<rmp_serde::encode::Error> for Error {
    fn from(err: rmp_serde::encode::Error) -> Error {
        Error::EncodeFailed(err)
    }
}

impl From<rmp_serde::decode::Error> for Error {
    fn from(err: rmp_serde::decode::Error) -> Error {
        Error::DecodeFailed(err)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Manifest {
    pub name: Option<String>,
    pub pieces: Vec<ManifestPieceRef>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct ManifestPieceRef {
    pub from: u64,
    pub len: u64,
    pub piece: PieceRef,
}

impl Manifest {
    pub fn for_file(path: &Path) -> io::Result<Manifest> {
        let mut file = File::open(path)?;
        let mut pieces = Vec::new();
        let mut pos = 0;
        loop {
            let mut buf = [0; PIECE_BYTES];
            let len = file.read(&mut buf)?;
            if len > 0 {
                let piece = PieceRef::for_buffer(&buf[..len]);
                pieces.push(ManifestPieceRef {
                    from: pos,
                    len: len as u64,
                    piece: piece,
                });
                pos += len as u64;
            } else {
                break;
            }
        }

        Ok(Manifest {
            name: path.file_name()
                .and_then(|x| x.to_str())
                .map(|x| x.to_string()),
            pieces: pieces,
        })
    }

    pub fn len(&self) -> u64 {
        self.pieces
            .iter()
            .map(|piece| piece.from + piece.len)
            .max()
            .unwrap_or(0)
    }

    pub fn to_vec(&self) -> Result<Vec<u8>, Error> {
        Ok(rmp_serde::to_vec(self)?)
    }

    pub fn from_read(read: &mut Read) -> Result<Manifest, Error> {
        Ok(rmp_serde::from_read(read)?)
    }
}

impl ManifestPieceRef {
    pub fn verify(&self, buf: &[u8]) -> bool {
        self.piece.verify(buf)
    }

    pub fn ref_vec(&self) -> Vec<u8> {
        self.piece.to_vec()
    }
}
