use std::fmt::Debug;

use crate::DecodeError;

#[derive(Debug)]
pub enum AddrRegister {
    BC,
    DE,
    HL,
    C,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Register16 {
    BC,
    DE,
    FG,
    HL,
    SP,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Register8 {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    L,
    H,
}

#[allow(dead_code)]
pub enum Slot {
    AddrRegister(AddrRegister),
    Register16(Register16),
    Register8(Register8),
    Addr8(u8),
    Addr16(u16),
    Data8(u8),
    Data16(u16),
}

impl Debug for Slot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Register8(val) => write!(f, "{:?}", val),
            Self::Register16(val) => write!(f, "{:?}", val),
            Self::AddrRegister(val) => write!(f, "({:?})", val),
            Self::Addr8(val) => write!(f, "(0x{:02x})", val),
            Self::Addr16(val) => write!(f, "(0x{:04x})", val),
            Self::Data8(val) => write!(f, "0x{:02x}", val),
            Self::Data16(val) => write!(f, "0x{:04x}", val),
        }
    }
}

impl Slot {
    #[allow(dead_code)]
    pub fn parse_a16(data: &mut impl Iterator<Item = u8>) -> Result<Self, DecodeError> {
        Ok(Slot::Addr16(decode_u16(data)?))
    }

    pub fn parse_a8(data: &mut impl Iterator<Item = u8>) -> Result<Self, DecodeError> {
        Ok(Slot::Addr8(decode_u8(data)?))
    }

    pub fn parse_d16(data: &mut impl Iterator<Item = u8>) -> Result<Self, DecodeError> {
        Ok(Slot::Data16(decode_u16(data)?))
    }

    pub fn parse_d8(data: &mut impl Iterator<Item = u8>) -> Result<Self, DecodeError> {
        Ok(Slot::Data8(decode_u8(data)?))
    }

    pub fn r8(r: Register8) -> Slot {
        Slot::Register8(r)
    }

    pub fn r16(r: Register16) -> Slot {
        Slot::Register16(r)
    }

    pub fn addr(r: AddrRegister) -> Slot {
        Slot::AddrRegister(r)
    }
}

fn decode_u8(data: &mut impl Iterator<Item = u8>) -> Result<u8, DecodeError> {
    data.next().ok_or(DecodeError::EndOfStream)
}

fn decode_u16(data: &mut impl Iterator<Item = u8>) -> Result<u16, DecodeError> {
    Ok(u16::from_le_bytes([
        data.next().ok_or(DecodeError::EndOfStream)?,
        data.next().ok_or(DecodeError::EndOfStream)?,
    ]))
}
