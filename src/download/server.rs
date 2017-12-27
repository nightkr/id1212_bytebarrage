use tokio_service::Service;
use tokio_proto::TcpServer;
use bytes::Bytes;
use futures::{future, Future};

use directory::Directory;
use piece::PieceRef;
use download::proto::DownloadProto;

use std::net::ToSocketAddrs;
use std::io::{self, Read, Seek, SeekFrom};
use std::fs::File;

struct DownloadServer {
    directory: Directory,
}

impl Service for DownloadServer {
    type Request = PieceRef;
    type Response = Bytes;
    type Error = io::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, piece_ref: Self::Request) -> Self::Future {
        Box::new(future::result(read_piece(&self.directory, &piece_ref)))
    }
}

fn read_piece(directory: &Directory, piece_ref: &PieceRef) -> io::Result<Bytes> {
    let dir_piece_ref = directory.find_piece(&piece_ref).ok_or(io::Error::new(
        io::ErrorKind::NotFound,
        format!(
            "unknown piece {:?}",
            piece_ref
        ),
    ))?;
    let mut file = File::open(dir_piece_ref.file)?;
    file.seek(SeekFrom::Start(dir_piece_ref.from))?;
    let mut buf = Vec::new();
    buf.resize(dir_piece_ref.len as usize, 0);
    file.read_exact(&mut buf)?;
    Ok(Bytes::from(buf))
}

pub fn listen<Addr: ToSocketAddrs>(addr: Addr, directory: &Directory) -> io::Result<()> {
    let server = TcpServer::new(DownloadProto, addr.to_socket_addrs()?.next().unwrap());
    let dir = directory.clone();
    server.serve(move || Ok(DownloadServer { directory: dir.clone() }));
    Ok(())
}
