use tokio_core::reactor::{Core, Handle};
use tokio_proto::TcpClient;
use tokio_service::Service;
use futures::future::{self, Future, Loop};

use manifest::{Manifest, ManifestPieceRef};
use download::proto::DownloadProto;

use std::collections::VecDeque;
use std::path::Path;
use std::fs::File;
use std::net::{TcpStream, ToSocketAddrs, SocketAddr};
use std::io::{self, Read, Seek, SeekFrom, Write};

fn download_pieces<'q>(addr: Box<SocketAddr>, handle: Box<Handle>, mut queue: VecDeque<ManifestPieceRef>, mut file: File) -> Box<Future<Item = Loop<(), (File, VecDeque<ManifestPieceRef>)>, Error = io::Error>> {
    if let Some(piece) = queue.pop_front() {
        println!("Downloading piece {:?}", piece);
        // let piece_ref = piece.piece;
        Box::new(TcpClient::new(DownloadProto).connect(&addr, &handle)
                 .and_then(move |client| {
                     client.call(piece)
                 })
                 .and_then(move |buf| {
                     file.seek(SeekFrom::Start(piece.from))?;
                     file.write_all(&buf)?;
                     Ok(Loop::Continue((file, queue)))
                 }))
    } else {
        Box::new(future::ok(Loop::Break(())))
    }
}

pub fn download<Addr: ToSocketAddrs>(
    addr: Addr,
    manifest: &Manifest,
    path: &Path,
) -> io::Result<()> {
    let address = addr.to_socket_addrs()?.next().unwrap();
    let mut core = Core::new()?;
    let file = File::create(path)?;
    let queue = VecDeque::from(manifest.pieces.clone());
    let handle = core.handle();
    core.run(future::loop_fn((file, queue), move |(file, queue)| download_pieces(Box::new(address), Box::new(handle.clone()), queue, file)))?;
    Ok(())
}
