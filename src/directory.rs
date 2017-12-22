use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{Error, Result};

use super::manifest::Manifest;
use super::piece::PieceRef;
use super::fs_walker::walk_dir_files;

#[derive(Debug)]
pub struct Directory {
    pieces: HashMap<PieceRef, DirectoryPieceRef>,
}

#[derive(Debug)]
pub struct DirectoryPieceRef {
    from: u64,
    len: u64,
    reference: PieceRef,
    file: PathBuf,
}

impl Directory {
    pub fn new() -> Directory {
        Directory {
            pieces: HashMap::default(),
        }
    }

    pub fn add_manifest(&mut self, path: &Path, manifest: &Manifest) {
        for piece in manifest.pieces().iter() {
            self.pieces.insert(
                piece.piece,
                DirectoryPieceRef {
                    from: piece.from,
                    len: piece.len,
                    reference: piece.piece,
                    file: path.to_owned(),
                },
            );
        }
    }

    pub fn scan_folder(&mut self, path: &Path) -> Result<()> {
        walk_dir_files::<Error, _>(path.to_owned(), &mut |file_path| {
            println!("Building manifest of {}", file_path.display());
            let manifest = Manifest::for_file(&file_path)?;
            self.add_manifest(&file_path, &manifest);
            Ok(())
        })?;
        Ok(())
    }
}
