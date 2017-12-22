use super::piece::PieceRef;

use std::path::Path;
use std::fs::File;
use std::io::{Read, Result};

const PIECE_BYTES: usize = 1024 * 1024;

#[derive(Debug)]
pub struct Manifest {
    pub name: Option<String>,
    pieces: Vec<ManifestPieceRef>,
}

#[derive(Debug)]
pub struct ManifestPieceRef {
    pub from: u64,
    pub len: u64,
    pub piece: PieceRef,
}

impl Manifest {
    pub fn for_file(path: &Path) -> Result<Manifest> {
        let mut file = File::open(path)?;
        let mut pieces = Vec::new();
        let mut pos = 0;
        loop {
            let mut buf = [0; PIECE_BYTES];
            let len = file.read(&mut buf)?;
            if len > 0 {
                let piece = PieceRef::for_buffer(&buf);
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

    pub fn pieces(&self) -> &[ManifestPieceRef] {
        &self.pieces
    }
}
