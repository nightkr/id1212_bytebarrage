use bytes::Bytes;
use tokio_io::codec::Framed;
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_proto::pipeline::{ClientProto, ServerProto};
use serde_bytes;

use piece::PieceRef;
use rmp_codec::RmpCodec;

use std::io;

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMsg {
    Query(PieceRef),
    Get(PieceRef),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMsg {
    QueryResult(bool),
    Contents(#[serde(with = "serde_bytes")] Bytes),
}

type ServerCodec = RmpCodec<ServerMsg, ClientMsg>;
type ClientCodec = RmpCodec<ClientMsg, ServerMsg>;

pub struct DownloadProto;

impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for DownloadProto {
    type Request = ServerMsg;
    type Response = ClientMsg;
    type Transport = Framed<T, ServerCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;
    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(ServerCodec::default()))
    }
}

impl<T: AsyncRead + AsyncWrite + 'static> ClientProto<T> for DownloadProto {
    type Request = ServerMsg;
    type Response = ClientMsg;
    type Transport = Framed<T, ClientCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;
    fn bind_transport(&self, io: T) -> Self::BindTransport {
        // Ok(io.framed(ClientCodec { curr_request: None }))
        Ok(io.framed(ClientCodec::default()))
    }
}
