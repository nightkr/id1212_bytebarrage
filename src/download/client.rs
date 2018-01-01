use tokio_core::reactor::{Core, Handle};
use tokio_proto::TcpClient;
use tokio_service::Service;
use tokio_timer::Timer;
use futures::future::{self, Future};

use manifest::{Manifest, ManifestPieceRef};
use super::proto::{ClientMsg, DownloadProto, ServerMsg};
use discovery::client::{listen as discovery_listen, DiscoveredPieces};

use std::collections::VecDeque;
use std::path::Path;
use std::fs::File;
use std::io::{self, Seek, SeekFrom, Write};
use std::time::Duration;

fn download_pieces<'f>(
    handle: &Handle,
    discovery_mgr: &'f DiscoveredPieces,
    timer: &'f Timer,
    queue: &'f mut VecDeque<ManifestPieceRef>,
    file: &'f mut File,
) -> Box<'f + Future<Item = (), Error = io::Error>> {
    if let Some(piece) = queue.pop_front() {
        println!("Downloading piece {:?}", piece);
        let piece_peers = discovery_mgr
            .get_piece_peers(&piece.piece)
            .into_iter()
            .map(|addr| {
                let discovery_mgr = discovery_mgr.clone();
                TcpClient::new(DownloadProto)
                    .connect(&addr, &handle)
                    .and_then(move |client| {
                        client
                            .call(ServerMsg::Query(piece.piece))
                            .and_then(move |msg| match msg {
                                ClientMsg::QueryResult(true) => Ok((client, addr)),
                                _ => {
                                    discovery_mgr.remove_piece_peer(&piece.piece, &addr);
                                    Err(io::Error::new(io::ErrorKind::NotFound, "piece not found"))
                                }
                            })
                    })
            })
            .collect::<Vec<_>>();

        let download: Box<'f + Future<Item = _, Error = _>> = if !piece_peers.is_empty() {
            Box::new(
                future::select_ok(piece_peers)
                    .and_then(move |((client, addr), _)| client.call(ServerMsg::Get(piece.piece)).map(move |msg| (msg, addr)))
                    .and_then(move |(msg, addr)| match msg {
                        ClientMsg::Contents(buf) => {
                            if piece.verify(&buf) {
                                file.seek(SeekFrom::Start(piece.from))?;
                                file.write_all(&buf)?;
                                Ok(())
                            } else {
                                println!("Received damaged piece {:?} from {}, blacklisting...", piece.piece, addr);
                                discovery_mgr.remove_piece_peer(&piece.piece, &addr);
                                Err(io::Error::new(io::ErrorKind::InvalidData, "piece validation failed"))
                            }
                        }
                        _ => Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            format!("{:?}", msg),
                        )),
                    }),
            )
        } else {
            Box::new(
                timer
                    .sleep(Duration::from_millis(500))
                    .then(|_| Err(io::Error::new(io::ErrorKind::NotFound, "no peer has piece"))),
            )
        };

        Box::new(download.then(move |result| match result {
            Ok(()) => Box::new(future::ok(())),
            Err(err) => {
                println!("Failed to download {:?}, deferring: {}", piece.piece, err);
                queue.push_back(piece);
                discovery_mgr.enqueue_piece(piece.piece);
                Box::new(future::ok(()))
            }
        }))
    } else {
        Box::new(future::ok(()))
    }
}

pub fn download(manifest: &Manifest, path: &Path) -> io::Result<()> {
    let mut core = Core::new()?;
    let timer = Timer::default();
    let mut file = File::create(path)?;
    let mut queue = VecDeque::from(manifest.pieces.clone());
    let handle = core.handle();
    let discovery_mgr = discovery_listen(&handle)?;
    for piece in queue.iter() {
        discovery_mgr.enqueue_piece(piece.piece);
    }
    while !queue.is_empty() {
        core.run(download_pieces(
            &handle,
            &discovery_mgr,
            &timer,
            &mut queue,
            &mut file,
        ))?;
    }
    Ok(())
}
