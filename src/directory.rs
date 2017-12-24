use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::io::{Error, Result};

use manifest::Manifest;
use piece::PieceRef;
use fs_walker::walk_dir_files;

#[derive(Debug, Clone)]
pub struct Directory {
    pieces: HashMap<PieceRef, DirectoryPieceRef>,
}

#[derive(Debug, Clone)]
pub struct DirectoryPieceRef {
    pub from: u64,
    pub len: u64,
    pub reference: PieceRef,
    pub file: PathBuf,
}

impl Directory {
    pub fn new() -> Directory {
        Directory {
            pieces: HashMap::default(),
        }
    }

    pub fn add_manifest(&mut self, path: &Path, manifest: &Manifest) {
        for piece in manifest.pieces.iter() {
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

    pub fn find_piece(&self, piece_ref: &PieceRef) -> Option<DirectoryPieceRef> {
        self.pieces.get(piece_ref).cloned()
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
