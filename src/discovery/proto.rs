use tokio_core::net::{UdpCodec, UdpFramed, UdpSocket};
use tokio_core::reactor::Handle;
use rmp_serde;
use net2::UdpBuilder;
#[cfg(unix)]
use net2::unix::UnixUdpBuilderExt;

use piece::PieceRef;

use std::{io, net};

fn discovery_addr() -> net::SocketAddr {
    net::SocketAddr::V4(net::SocketAddrV4::new(
        net::Ipv4Addr::new(255, 255, 255, 255),
        36936,
    ))
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Msg {
    Find(PieceRef),
    ServerHas(ServerPieceRef),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerPieceRef {
    pub addrs: Vec<net::SocketAddr>,
    pub piece: PieceRef,
}

pub struct Codec;

impl UdpCodec for Codec {
    type In = Msg;
    type Out = Msg;

    fn decode(&mut self, _src: &net::SocketAddr, buf: &[u8]) -> io::Result<Msg> {
        rmp_serde::from_slice(&buf).map_err(|err| io::Error::new(io::ErrorKind::Other, err))
    }

    fn encode(&mut self, msg: Msg, buf: &mut Vec<u8>) -> net::SocketAddr {
        buf.extend(rmp_serde::to_vec(&msg).unwrap());
        discovery_addr()
    }
}

pub fn bind(handle: &Handle) -> io::Result<UdpFramed<Codec>> {
    let builder = UdpBuilder::new_v4()?;
    bind_allow_port_reuse(&builder)?;
    let socket = UdpSocket::from_socket(builder.bind(&discovery_addr())?, handle)?;
    socket.set_broadcast(true)?;
    // let socket = UdpSocket::bind(&discovery_addr(), handle)?;
    Ok(socket.framed(Codec))
}

#[cfg(unix)]
fn bind_allow_port_reuse(builder: &UdpBuilder) -> io::Result<&UdpBuilder> {
    builder.reuse_address(true)?.reuse_port(true)
}

#[cfg(windows)]
fn bind_allow_port_reuse(builder: &UdpBuilder) -> io::Result<&UdpBuilder> {
    // reuse_port implied by reuse_address on Windows according to https://stackoverflow.com/a/14388707
    builder.reuse_address(true)
}
