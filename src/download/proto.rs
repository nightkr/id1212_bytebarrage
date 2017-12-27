use bytes::{Bytes, BytesMut};
use tokio_io::codec::{Decoder, Encoder, Framed};
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_proto::pipeline::{ServerProto, ClientProto};

use piece::PieceRef;

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

pub struct ClientCodec;

impl Decoder for ClientCodec {
    type Item = Bytes;
    type Error = io::Error;

    fn decode(&mut self, _buf: &mut BytesMut) -> io::Result<Option<Bytes>> {
        Ok(None)
    }

    fn decode_eof(&mut self, buf: &mut BytesMut) -> io::Result<Option<Bytes>> {
        if buf.is_empty() {
            Ok(None)
        } else {
            Ok(Some(buf.take().freeze()))
        }
    }
}

impl Encoder for ClientCodec {
    type Item = PieceRef;
    type Error = io::Error;

    fn encode(&mut self, msg: PieceRef, buf: &mut BytesMut) -> io::Result<()> {
        buf.extend(msg.to_vec());
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
    type Request = PieceRef;
    type Response = Bytes;
    type Transport = Framed<T, ClientCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;
    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(ClientCodec))
    }
}
