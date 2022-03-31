use crate::error::{PacketError, PacketProtocolError};
use byteorder::{ByteOrder, LittleEndian};
use derive_more::{Display, From, Into, IsVariant, Unwrap, UpperHex};
use electricui_embedded::prelude::*;
use ordered_float::OrderedFloat;
use std::{fmt, str};

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Into)]
pub struct OwnedMessageId(Vec<u8>);

impl OwnedMessageId {
    pub fn new(id: &[u8]) -> Option<Self> {
        MessageId::new(id).map(|id| Self::from_wire(&id))
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn as_str(&self) -> Result<&str, str::Utf8Error> {
        str::from_utf8(&self.0)
    }

    pub fn from_utf8(s: &str) -> Self {
        Self(s.as_bytes().to_vec())
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn as_wire(&self) -> MessageId<'_> {
        unsafe { MessageId::new_unchecked(&self.0) }
    }

    pub fn from_wire(id: &MessageId<'_>) -> Self {
        Self(id.as_bytes().to_vec())
    }
}

impl fmt::Display for OwnedMessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Ok(s) = self.as_str() {
            f.write_str(s)
        } else {
            write!(f, "{:X?}", self.0)
        }
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, IsVariant, Unwrap, Display)]
pub enum VariableKind {
    #[display(fmt = "Callback")]
    Callback,
    #[display(fmt = "Custom({:02X?})", _0)]
    Custom(Vec<u8>),
    #[display(fmt = "Unknown(0x{:02X} : {:02X?})", _0, _1)]
    Unknown(u8, Vec<u8>),
    #[display(fmt = "Byte(0x{:02X})", _0)]
    Byte(u8),
    #[display(fmt = "ByteArray({:02X?})", _0)]
    ByteArray(Vec<u8>),
    #[display(fmt = "Char({})", _0)]
    Char(char),
    #[display(fmt = "CharArray({})", _0)]
    CharArray(String),
    #[display(fmt = "I8({})", _0)]
    I8(i8),
    #[display(fmt = "I8Array({:?})", _0)]
    I8Array(Vec<i8>),
    #[display(fmt = "U8({})", _0)]
    U8(u8),
    #[display(fmt = "U8Array({:?})", _0)]
    U8Array(Vec<u8>),
    #[display(fmt = "I16({})", _0)]
    I16(i16),
    #[display(fmt = "I16Array({:?})", _0)]
    I16Array(Vec<i16>),
    #[display(fmt = "U16({})", _0)]
    U16(u16),
    #[display(fmt = "U16Array({:?})", _0)]
    U16Array(Vec<u16>),
    #[display(fmt = "I32({})", _0)]
    I32(i32),
    #[display(fmt = "I32Array({:?})", _0)]
    I32Array(Vec<i32>),
    #[display(fmt = "U32({})", _0)]
    U32(u32),
    #[display(fmt = "U32Array({:?})", _0)]
    U32Array(Vec<u32>),
    #[display(fmt = "F32({})", _0)]
    F32(OrderedFloat<f32>),
    #[display(fmt = "F32Array({:?})", _0)]
    F32Array(Vec<OrderedFloat<f32>>),
    #[display(fmt = "F64({})", _0)]
    F64(OrderedFloat<f64>),
    #[display(fmt = "F64Array({:?})", _0)]
    F64Array(Vec<OrderedFloat<f64>>),
}

impl VariableKind {
    pub fn from_wire(typ: MessageType, data: &[u8]) -> Result<Self, PacketProtocolError> {
        // TODO - protocol sanity checks, size checks, etc
        let num_elements = typ.array_wire_length_hint(data.len());
        let expected_size = typ.array_wire_size_hint(num_elements);
        if typ.wire_size_hint() != 0 && (expected_size != data.len()) {
            return Err(PacketProtocolError::ProtocolViolation);
        }
        // TODO check expected_size == data.len() on types that need it
        // err if num_elements == 0 on types that need it
        let is_array = num_elements > 1;
        Ok(match typ {
            MessageType::Callback => VariableKind::Callback,
            MessageType::Custom => VariableKind::Custom(data.to_vec()),
            MessageType::Unknown(t) => VariableKind::Unknown(t, data.to_vec()),
            // TODO - protocol still doesn't support offset packets, just throw them into unknown
            // for now
            MessageType::OffsetMetadata => VariableKind::Unknown(typ.into(), Default::default()),
            MessageType::Byte => {
                if is_array {
                    VariableKind::ByteArray(data.to_vec())
                } else {
                    VariableKind::Byte(data[0])
                }
            }
            MessageType::Char => {
                if is_array {
                    VariableKind::CharArray(str::from_utf8(data)?.to_string())
                } else {
                    VariableKind::Char(data[0] as _)
                }
            }
            MessageType::I8 => {
                if is_array {
                    VariableKind::I8Array(data.iter().map(|b| *b as _).collect())
                } else {
                    VariableKind::I8(data[0] as _)
                }
            }
            MessageType::U8 => {
                if is_array {
                    VariableKind::U8Array(data.to_vec())
                } else {
                    VariableKind::U8(data[0])
                }
            }
            MessageType::I16 => {
                if is_array {
                    let mut dst = vec![0; num_elements];
                    LittleEndian::read_i16_into(data, &mut dst);
                    VariableKind::I16Array(dst)
                } else {
                    VariableKind::I16(LittleEndian::read_i16(data))
                }
            }
            MessageType::U16 => {
                if is_array {
                    let mut dst = vec![0; num_elements];
                    LittleEndian::read_u16_into(data, &mut dst);
                    VariableKind::U16Array(dst)
                } else {
                    VariableKind::U16(LittleEndian::read_u16(data))
                }
            }
            MessageType::I32 => {
                if is_array {
                    let mut dst = vec![0; num_elements];
                    LittleEndian::read_i32_into(data, &mut dst);
                    VariableKind::I32Array(dst)
                } else {
                    VariableKind::I32(LittleEndian::read_i32(data))
                }
            }
            MessageType::U32 => {
                if is_array {
                    let mut dst = vec![0; num_elements];
                    LittleEndian::read_u32_into(data, &mut dst);
                    VariableKind::U32Array(dst)
                } else {
                    VariableKind::U32(LittleEndian::read_u32(data))
                }
            }
            MessageType::F32 => {
                if is_array {
                    let mut dst = vec![0.0; num_elements];
                    LittleEndian::read_f32_into(data, &mut dst);
                    VariableKind::F32Array(dst.into_iter().map(|f| f.into()).collect())
                } else {
                    VariableKind::F32(LittleEndian::read_f32(data).into())
                }
            }
            MessageType::F64 => {
                if is_array {
                    let mut dst = vec![0.0; num_elements];
                    LittleEndian::read_f64_into(data, &mut dst);
                    VariableKind::F64Array(dst.into_iter().map(|f| f.into()).collect())
                } else {
                    VariableKind::F64(LittleEndian::read_f64(data).into())
                }
            }
        })
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Display)]
#[display(fmt = "Id({}), Kind({})", id, kind)]
pub struct Variable {
    pub id: OwnedMessageId,
    pub kind: VariableKind,
}

// TODO zero is invalid, no From
#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Display, UpperHex, From, Into,
)]
pub struct BoardId(u16);

impl BoardId {
    pub fn encode_request<T: AsRef<[u8]> + AsMut<[u8]>>(
        p: &mut Packet<T>,
    ) -> Result<(), PacketError> {
        p.set_data_length(0)?;
        p.set_typ(MessageType::U16);
        p.set_internal(true);
        p.set_offset(false);
        p.set_id_length(MessageId::INTERNAL_BOARD_ID.len() as _)?;
        p.set_response(true);
        p.set_acknum(0);
        p.msg_id_mut()?
            .copy_from_slice(MessageId::INTERNAL_BOARD_ID.as_bytes());
        p.set_checksum(p.compute_checksum()?)?;
        Ok(())
    }

    pub fn decode_response<T: AsRef<[u8]>>(p: &Packet<T>) -> Result<Self, PacketProtocolError> {
        // TODO - sanity check protocol
        let id = p.payload()?;
        Ok(LittleEndian::read_u16(id).into())
    }
}

// TODO - empty is invalid
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, From, Into)]
pub struct BoardName(Vec<u8>);

impl BoardName {
    pub fn as_str(&self) -> Result<&str, str::Utf8Error> {
        str::from_utf8(&self.0)
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn encode_request<T: AsRef<[u8]> + AsMut<[u8]>>(
        p: &mut Packet<T>,
    ) -> Result<(), PacketError> {
        p.set_data_length(0)?;
        p.set_typ(MessageType::Callback);
        p.set_internal(false);
        p.set_offset(false);
        p.set_id_length(MessageId::BOARD_NAME.len() as _)?;
        p.set_response(true);
        p.set_acknum(0);
        p.msg_id_mut()?
            .copy_from_slice(MessageId::BOARD_NAME.as_bytes());
        p.set_checksum(p.compute_checksum()?)?;
        Ok(())
    }

    pub fn decode_response<T: AsRef<[u8]>>(p: &Packet<T>) -> Result<Self, PacketProtocolError> {
        // TODO - sanity check protocol
        let name = p.payload()?;
        Ok(name.to_vec().into())
    }
}

impl fmt::Display for BoardName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Ok(s) = self.as_str() {
            f.write_str(s)
        } else {
            write!(f, "{:X?}", self.0)
        }
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct WritableIdsAnnouncement;

impl WritableIdsAnnouncement {
    pub fn encode_request<T: AsRef<[u8]> + AsMut<[u8]>>(
        p: &mut Packet<T>,
    ) -> Result<(), PacketError> {
        p.set_data_length(0)?;
        p.set_typ(MessageType::Callback);
        p.set_internal(true);
        p.set_offset(false);
        p.set_id_length(1)?;
        p.set_response(true);
        p.set_acknum(0);
        p.msg_id_mut()?
            .copy_from_slice(MessageId::INTERNAL_AM.as_bytes());
        p.set_checksum(p.compute_checksum()?)?;
        Ok(())
    }
}

// TODO - allow accumulating
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, From, Into)]
pub struct IdsAnnouncement(Vec<OwnedMessageId>);

impl IdsAnnouncement {
    pub fn as_slice(&self) -> &[OwnedMessageId] {
        &self.0
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn decode_response<T: AsRef<[u8]>>(p: &Packet<T>) -> Result<Self, PacketProtocolError> {
        // TODO - sanity check protocol
        let ids: Vec<OwnedMessageId> = p
            .payload()?
            .split(|b| *b == b'\0')
            .filter_map(OwnedMessageId::new)
            .collect();
        Ok(Self(ids))
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Into)]
pub struct WritableIdsAnnouncementEndList(usize);

impl WritableIdsAnnouncementEndList {
    pub fn decode_response<T: AsRef<[u8]>>(p: &Packet<T>) -> Result<Self, PacketProtocolError> {
        // TODO - sanity check protocol
        let num_ids = p.payload()?;
        Ok(Self(if p.typ() == MessageType::U8 {
            num_ids[0] as usize
        } else {
            // u16
            LittleEndian::read_u16(num_ids) as usize
        }))
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Into)]
pub struct TrackedVariables(Vec<Variable>);

impl TrackedVariables {
    pub fn as_slice(&self) -> &[Variable] {
        &self.0
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn encode_request<T: AsRef<[u8]> + AsMut<[u8]>>(
        p: &mut Packet<T>,
    ) -> Result<(), PacketError> {
        p.set_data_length(0)?;
        p.set_typ(MessageType::Callback);
        p.set_internal(true);
        p.set_offset(false);
        p.set_id_length(MessageId::INTERNAL_AV.len() as _)?;
        p.set_response(true);
        p.set_acknum(0);
        p.msg_id_mut()?
            .copy_from_slice(MessageId::INTERNAL_AV.as_bytes());
        p.set_checksum(p.compute_checksum()?)?;
        Ok(())
    }

    // TODO - accumulating here or alt pattern
    pub fn decode_response_accumulating<T: AsRef<[u8]>>(
        &mut self,
        p: &Packet<T>,
    ) -> Result<(), PacketProtocolError> {
        // TODO - sanity check protocol
        let id = p.msg_id()?;
        let typ = p.typ();
        let data = p.payload()?;
        self.0.push(Variable {
            id: OwnedMessageId::from_wire(&id),
            kind: VariableKind::from_wire(typ, data)?,
        });
        Ok(())
    }
}

#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Display, UpperHex, From, Into,
)]
pub struct Heartbeat(u8);

impl Heartbeat {
    pub fn encode_request<T: AsRef<[u8]> + AsMut<[u8]>>(
        &self,
        p: &mut Packet<T>,
    ) -> Result<(), PacketError> {
        p.set_data_length(1)?;
        p.set_typ(MessageType::U8);
        p.set_internal(true);
        p.set_offset(false);
        p.set_id_length(MessageId::INTERNAL_HEARTBEAT.len() as _)?;
        p.set_response(true);
        p.set_acknum(0);
        p.msg_id_mut()?
            .copy_from_slice(MessageId::INTERNAL_HEARTBEAT.as_bytes());
        p.payload_mut()?[0] = self.0;
        p.set_checksum(p.compute_checksum()?)?;
        Ok(())
    }

    pub fn decode_response<T: AsRef<[u8]>>(p: &Packet<T>) -> Result<Self, PacketProtocolError> {
        // TODO - sanity check protocol
        let hb = p.payload()?;
        Ok(hb[0].into())
    }
}
