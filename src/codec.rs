use bytes::{Buf, Bytes, BytesMut};
use electricui_embedded::{decoder, wire};
use std::io;
use thiserror::Error;
use tokio_util::codec;
use tracing::{debug, warn};

// TODO - need a std Error impl for the eui types
#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Decoder(#[from] DecoderError),

    #[error(transparent)]
    Encoder(#[from] EncoderError),

    #[error("Encountered in IO error while encoding/decoding.")]
    Io(#[from] io::Error),
}

pub struct Codec<'buf, const N: usize> {
    dec: Decoder<'buf, N>,
    enc: Encoder,
}

impl<'buf, const N: usize> Codec<'buf, N> {
    pub fn new(dec: decoder::Decoder<'buf, N>) -> Self {
        Self {
            dec: Decoder::new(dec),
            enc: Encoder::default(),
        }
    }
}

impl<'buf, const N: usize> codec::Decoder for Codec<'buf, N> {
    // TODO - figure out the lifetime shenanigans and return borrowed
    // content from decoder instead of creating a new Bytes packet
    //type Item = wire::Packet<&'a [u8]>;
    type Item = wire::Packet<Bytes>;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let pkt = self.dec.decode(src)?;
        Ok(pkt)
    }
}

impl<'buf, const N: usize, T: AsRef<[u8]>> codec::Encoder<wire::Packet<T>> for Codec<'buf, N> {
    type Error = Error;

    fn encode(&mut self, item: wire::Packet<T>, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.enc.encode(item, dst)?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum DecoderError {
    #[error(transparent)]
    EUiDecoder(#[from] crate::error::DecoderError),

    #[error("Encountered in IO error while decoding.")]
    Io(#[from] io::Error),
}

#[derive(Debug)]
pub struct Decoder<'buf, const N: usize> {
    dec: decoder::Decoder<'buf, N>,
}

impl<'buf, const N: usize> Decoder<'buf, N> {
    pub fn new(dec: decoder::Decoder<'buf, N>) -> Self {
        Self { dec }
    }
}

impl<'buf, const N: usize> codec::Decoder for Decoder<'buf, N> {
    // TODO - figure out the lifetime shenanigans and return borrowed
    // content from decoder instead of creating a new Bytes packet
    //type Item = wire::Packet<&'a [u8]>;
    type Item = wire::Packet<Bytes>;
    type Error = DecoderError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        for idx in 0..src.len() {
            match self
                .dec
                .decode(src[idx])
                .map_err(crate::error::DecoderError)
            {
                Err(e) => {
                    warn!("Discarding {} bytes due to decoder error", idx);
                    let mut junk = src.split_to(idx);
                    junk.clear();
                    // TODO - advance or clear
                    return Err(e.into());
                }
                Ok(None) => (),
                Ok(Some(pkt)) => {
                    debug!("Found packet size={}, {}", pkt.as_ref().len(), pkt);
                    // TODO - advance or clear
                    src.advance(idx + 1);
                    return Ok(Some(wire::Packet::new_unchecked(Bytes::copy_from_slice(
                        pkt.as_ref(),
                    ))));
                }
            }
        }
        src.clear();
        Ok(None)
    }
}

#[derive(Debug, Error)]
pub enum EncoderError {
    #[error(transparent)]
    EUiPacket(#[from] crate::error::PacketError),

    #[error("Encountered in IO error while encoding.")]
    Io(#[from] io::Error),
}

#[derive(Debug, Default)]
pub struct Encoder {}

impl<T: AsRef<[u8]>> codec::Encoder<wire::Packet<T>> for Encoder {
    type Error = EncoderError;

    fn encode(&mut self, item: wire::Packet<T>, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let pkt_size = item.wire_size().map_err(crate::error::PacketError)?;
        dst.resize(wire::Framing::max_encoded_len(pkt_size), 0);
        let wire_size = wire::Framing::encode_buf(&item.as_ref()[..pkt_size], dst);
        dst.resize(wire_size, 0);
        debug!("Encoding {item}, wire_size={wire_size}");
        Ok(())
    }
}
