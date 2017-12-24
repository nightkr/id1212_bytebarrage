use directory::Directory;
use piece::PieceRef;

use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::fs::File;
use std::thread;

fn handle_request(directory: &Directory, mut stream: TcpStream) -> io::Result<()> {
    let piece_ref = PieceRef::from_read(&mut stream)?;
    let dir_piece_ref = directory.find_piece(&piece_ref).ok_or(io::Error::new(
        io::ErrorKind::NotFound,
        format!("unknown piece {:?}", piece_ref),
    ))?;
    let mut file = File::open(dir_piece_ref.file)?;
    file.seek(SeekFrom::Start(dir_piece_ref.from))?;
    let mut buf = Vec::with_capacity(dir_piece_ref.len as usize);
    buf.resize(dir_piece_ref.len as usize, 0);
    file.read_exact(&mut buf)?;
    stream.write_all(&mut buf)?;
    stream.flush()?;
    Ok(())
}

pub fn listen<Addr: ToSocketAddrs>(addr: Addr, directory: &Directory) -> io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    loop {
        let (stream, addr) = listener.accept()?;
        println!("New connection from {}", addr);
        let dir = directory.clone();
        thread::spawn(move || {
            handle_request(&dir, stream).unwrap();
        });
    }
}
