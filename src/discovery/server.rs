use tokio_core::reactor::Handle;
use pnet_datalink;
use futures::{Future, Stream};

use super::proto::{bind, Msg, ServerPieceRef};
use directory::Directory;

use std::io;
use std::net::SocketAddr;

fn local_addrs(port: u16) -> Vec<SocketAddr> {
    pnet_datalink::interfaces()
        .into_iter()
        .flat_map(|iface| iface.ips)
        .map(|addr| addr.ip())
        .map(|ip| SocketAddr::new(ip, port))
        .collect()
}

fn handle_packet(directory: &Directory, addrs: &[SocketAddr], msg: Msg) -> Option<Msg> {
    match msg {
        Msg::Find(piece) => {
            if directory.find_piece(&piece).is_some() {
                Some(Msg::ServerHas(ServerPieceRef {
                    addrs: addrs.to_vec(),
                    piece: piece,
                }))
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn listen(handle: &Handle, port: u16, directory: Directory) -> io::Result<()> {
    let (sock_sink, sock_stream) = bind(handle)?.split();
    let addrs = local_addrs(port);
    handle.spawn(
        sock_stream
            .filter_map(move |msg| handle_packet(&directory, &addrs, msg))
            .forward(sock_sink)
            .then(|x| {
                x.unwrap();
                Ok(())
            }),
    );
    Ok(())
}
