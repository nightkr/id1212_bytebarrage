use bytes::{Bytes, BytesMut};
use tokio_io::codec::{Decoder, Encoder, Framed};
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_proto::pipeline::{ServerProto, ClientProto};

use piece::PieceRef;
use manifest::ManifestPieceRef;

use std::io;

pub struct ServerCodec;

impl Decoder for ServerCodec {
    type Item = PieceRef;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<PieceRef>> {
        Ok(PieceRef::from_bytes_mut(buf))
    }
}

impl Encoder for ServerCodec {
    type Item = Bytes;
    type Error = io::Error;

    fn encode(&mut self, msg: Bytes, buf: &mut BytesMut) -> io::Result<()> {
        buf.extend(msg);
        Ok(())
    }
}

pub struct ClientCodec {
    curr_request: Option<ManifestPieceRef>,
}

impl Decoder for ClientCodec {
    type Item = Bytes;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<Bytes>> {
        if let Some(piece) = self.curr_request {
            let len = piece.len as usize;
            if buf.len() >= len {
                self.curr_request = None;
                Ok(Some(buf.split_to(len).freeze()))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}

impl Encoder for ClientCodec {
    type Item = ManifestPieceRef;
    type Error = io::Error;

    fn encode(&mut self, msg: ManifestPieceRef, buf: &mut BytesMut) -> io::Result<()> {
        self.curr_request = Some(msg);
        buf.extend(msg.piece.to_vec());
        Ok(())
    }
}

pub struct DownloadProto;

impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for DownloadProto {
    type Request = PieceRef;
    type Response = Bytes;
    type Transport = Framed<T, ServerCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;
    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(ServerCodec))
    }
}

impl<T: AsyncRead + AsyncWrite + 'static> ClientProto<T> for DownloadProto {
    type Request = ManifestPieceRef;
    type Response = Bytes;
    type Transport = Framed<T, ClientCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;
    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(ClientCodec {
            curr_request: None,
        }))
    }
}
