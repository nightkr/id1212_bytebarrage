use bytes::BytesMut;
use serde::{Deserialize, Serialize};
use tokio_io::codec::{Decoder, Encoder};
use rmp_serde;

use std::io::{self, Cursor};
use std::marker::PhantomData;

pub struct RmpCodec<In, Out> {
    in_dummy: PhantomData<In>,
    out_dummy: PhantomData<Out>,
}

impl<In, Out> Default for RmpCodec<In, Out> {
    fn default() -> RmpCodec<In, Out> {
        RmpCodec {
            in_dummy: Default::default(),
            out_dummy: Default::default(),
        }
    }
}

impl<In, Out: Serialize> Encoder for RmpCodec<In, Out> {
    type Item = Out;
    type Error = io::Error;

    fn encode(&mut self, item: Out, buf: &mut BytesMut) -> io::Result<()> {
        let vec = rmp_serde::to_vec(&item)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        buf.extend(vec);
        Ok(())
    }
}

impl<In: for<'de> Deserialize<'de>, Out> Decoder for RmpCodec<In, Out> {
    type Item = In;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<In>> {
        let (msg, pos) = {
            let mut cursor = Cursor::new(&buf);
            let msg: In = match rmp_serde::from_read(&mut cursor) {
                Ok(x) => x,
                // Not enough data, read more and try again
                Err(rmp_serde::decode::Error::InvalidDataRead(_err))
                | Err(rmp_serde::decode::Error::InvalidMarkerRead(_err)) => return Ok(None),
                Err(err) => return Err(io::Error::new(io::ErrorKind::InvalidInput, err)),
            };
            (msg, cursor.position())
        };
        buf.split_to(pos as usize);
        Ok(Some(msg))
    }
}
