use tokio_core::reactor::{Handle, Interval};
use futures::{Future, Stream};

use piece::PieceRef;
use super::proto::{bind, Msg};

use std::collections::{HashMap, HashSet, VecDeque};
use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug)]
struct PieceInfo {
    peers: HashSet<SocketAddr>,
    blacklist: HashSet<SocketAddr>,
}

impl PieceInfo {
    fn new() -> PieceInfo {
        PieceInfo {
            peers: HashSet::default(),
            blacklist: HashSet::default(),
        }
    }

    fn add_peer(&mut self, addrs: &[SocketAddr]) {
        let blacklist = self.blacklist.clone();
        self.peers
            .extend(addrs.iter().filter(|addr| !blacklist.contains(addr)));
    }

    fn blacklist_peer(&mut self, addr: &SocketAddr) {
        self.peers.remove(addr);
        self.blacklist.insert(*addr);
    }

    fn peers(&self) -> &HashSet<SocketAddr> {
        &self.peers
    }
}

#[derive(Debug)]
struct DiscoveredPiecesInner {
    pieces: HashMap<PieceRef, PieceInfo>,
    queue: VecDeque<PieceRef>,
}

impl DiscoveredPiecesInner {
    fn new() -> DiscoveredPiecesInner {
        DiscoveredPiecesInner {
            pieces: HashMap::default(),
            queue: VecDeque::default(),
        }
    }

    fn get_piece(&self, piece_ref: &PieceRef) -> Option<&PieceInfo> {
        self.pieces.get(piece_ref)
    }

    fn get_piece_mut(&mut self, piece_ref: &PieceRef) -> &mut PieceInfo {
        self.pieces.entry(*piece_ref).or_insert_with(PieceInfo::new)
    }
}

#[derive(Debug, Clone)]
pub struct DiscoveredPieces {
    inner: Arc<Mutex<DiscoveredPiecesInner>>,
}

impl DiscoveredPieces {
    fn new() -> DiscoveredPieces {
        DiscoveredPieces {
            inner: Arc::new(Mutex::new(DiscoveredPiecesInner::new())),
        }
    }

    fn add_piece_peer(&self, piece: &PieceRef, addrs: &[SocketAddr]) {
        let mut inner = self.inner.lock().unwrap();
        inner.get_piece_mut(piece).add_peer(addrs);
    }

    pub fn remove_piece_peer(&self, piece: &PieceRef, addr: &SocketAddr) {
        let mut inner = self.inner.lock().unwrap();
        inner.get_piece_mut(piece).blacklist_peer(addr);
    }

    fn dequeue_piece(&self) -> Option<PieceRef> {
        let mut inner = self.inner.lock().unwrap();
        inner.queue.pop_front()
    }

    pub fn enqueue_piece(&self, piece: PieceRef) {
        let mut inner = self.inner.lock().unwrap();
        inner.queue.push_back(piece)
    }

    pub fn get_piece_peers(&self, piece: &PieceRef) -> Vec<SocketAddr> {
        let inner = self.inner.lock().unwrap();
        if let Some(piece) = inner.get_piece(piece) {
            piece.peers().iter().map(Clone::clone).collect()
        } else {
            vec![]
        }
    }
}

fn query_pieces(manager: DiscoveredPieces) -> Option<Msg> {
    manager.dequeue_piece().map(Msg::Find)
}

fn handle_packet(manager: DiscoveredPieces, msg: Msg) {
    match msg {
        Msg::ServerHas(piece_info) => manager.add_piece_peer(&piece_info.piece, &piece_info.addrs),
        _ => {}
    }
}

pub fn listen(handle: &Handle) -> io::Result<DiscoveredPieces> {
    let manager = DiscoveredPieces::new();
    let (sock_sink, sock_stream) = bind(handle)?.split();
    let broadcast_mgr = manager.clone();
    handle.spawn(
        Interval::new(Duration::from_millis(5), handle)?
            .filter_map(move |()| query_pieces(broadcast_mgr.clone()))
            .forward(sock_sink)
            .then(|x| {
                x.unwrap();
                Ok(())
            }),
    );
    let recieve_mgr = manager.clone();
    handle.spawn(
        sock_stream
            .for_each(move |msg| Ok(handle_packet(recieve_mgr.clone(), msg)))
            .then(|x| {
                x.unwrap();
                Ok(())
            }),
    );
    Ok(manager)
}
