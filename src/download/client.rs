use manifest::Manifest;

use std::path::Path;
use std::fs::File;
use std::net::{TcpStream, ToSocketAddrs};
use std::io::{self, Read, Seek, SeekFrom, Write};

pub fn download<Addr: ToSocketAddrs>(
    addr: Addr,
    manifest: &Manifest,
    path: &Path,
) -> io::Result<()> {
    let mut file = File::create(path)?;
    for piece in manifest.pieces.iter() {
        println!("downloading piece {:?}", piece);
        let mut stream = TcpStream::connect(&addr)?;
        stream.write_all(&piece.ref_vec())?;
        let mut buf = Vec::with_capacity(piece.len as usize);
        buf.resize(piece.len as usize, 0);
        stream.read_exact(&mut buf)?;
        if !piece.verify(&buf) {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("failed to validate piece {:?}", piece)));
        }
        file.seek(SeekFrom::Start(piece.from))?;
        file.write_all(&buf)?;
    }
    Ok(())
}
