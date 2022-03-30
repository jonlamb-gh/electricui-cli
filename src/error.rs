use derive_more::From;
use std::{error, fmt, str};
use thiserror::Error;

#[derive(Debug, Copy, Clone, Eq, PartialEq, From)]
pub struct PacketError(pub electricui_embedded::wire::packet::Error);

impl error::Error for PacketError {}

impl fmt::Display for PacketError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Copy, Clone, From)]
pub struct FramingError(pub electricui_embedded::wire::framing::Error);

impl error::Error for FramingError {}

impl fmt::Display for FramingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, From)]
pub struct DecoderError(pub electricui_embedded::decoder::Error);

impl error::Error for DecoderError {}

impl fmt::Display for DecoderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum PacketProtocolError {
    #[error(transparent)]
    Packet(#[from] PacketError),

    #[error("Character array contains non-UTF-8 data")]
    Utf8(#[from] str::Utf8Error),

    #[error("Packet contains a protocol violation")]
    ProtocolViolation,
}

impl From<electricui_embedded::wire::packet::Error> for PacketProtocolError {
    fn from(e: electricui_embedded::wire::packet::Error) -> Self {
        PacketError(e).into()
    }
}
