use tokio_service::Service;
use tokio_proto::TcpServer;
use bytes::Bytes;
use futures::{future, Future};

use directory::Directory;
use piece::PieceRef;
use super::proto::{ClientMsg, DownloadProto, ServerMsg};
use discovery;

use std::net::ToSocketAddrs;
use std::io::{self, Read, Seek, SeekFrom};
use std::fs::File;

struct DownloadServer {
    directory: Directory,
}

impl Service for DownloadServer {
    type Request = ServerMsg;
    type Response = ClientMsg;
    type Error = io::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        match req {
            ServerMsg::Get(piece_ref) => Box::new(future::result(
                read_piece(&self.directory, &piece_ref).map(
                    ClientMsg::Contents,
                ),
            )),
            ServerMsg::Query(piece_ref) => Box::new(future::ok(ClientMsg::QueryResult(
                self.directory.find_piece(&piece_ref).is_some(),
            ))),
        }
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
    if file.metadata()?.len() >= dir_piece_ref.from + dir_piece_ref.len {
        buf.resize(dir_piece_ref.len as usize, 0);
        file.read_exact(&mut buf)?;
    } else {
        // Oh dear, the file has shrunk...
        // If we send an empty buffer then the client should reject it and retry
    }
    Ok(Bytes::from(buf))
}

pub fn listen<Addr: ToSocketAddrs>(addrs: Addr, directory: &Directory) -> io::Result<()> {
    let addr = addrs.to_socket_addrs()?.next().unwrap();
    let server = TcpServer::new(DownloadProto, addr);
    let dir = directory.clone();
    server.with_handle(move |handle| {
        let dir = dir.clone();
        discovery::server::listen(handle, addr.port(), dir.clone()).unwrap();
        move || Ok(DownloadServer { directory: dir.clone() })
    });
    Ok(())
}
