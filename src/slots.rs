use std::fmt::Debug;

use crate::DecodeError;

#[derive(Debug, PartialEq)]
pub enum AddrRegister {
    BC,
    DE,
    HL,
    C,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub enum Register16 {
    AF,
    BC,
    DE,
    FG,
    HL,
    SP,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
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
#[derive(PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slot_parse_a16() {
        let mut data = vec![0x12, 0x34].into_iter();
        assert_eq!(Slot::parse_a16(&mut data), Ok(Slot::Addr16(0x3412)));
        assert_eq!(Slot::parse_a16(&mut data), Err(DecodeError::EndOfStream));
    }

    #[test]
    fn test_slot_parse_a8() {
        let mut data = vec![0x12].into_iter();
        assert_eq!(Slot::parse_a8(&mut data), Ok(Slot::Addr8(0x12)));
        assert_eq!(Slot::parse_a8(&mut data), Err(DecodeError::EndOfStream));
    }

    #[test]
    fn test_slot_parse_d16() {
        let mut data = vec![0x12, 0x34].into_iter();
        assert_eq!(Slot::parse_d16(&mut data), Ok(Slot::Data16(0x3412)));
        assert_eq!(Slot::parse_d16(&mut data), Err(DecodeError::EndOfStream));
    }

    #[test]
    fn test_slot_parse_d8() {
        let mut data = vec![0x12].into_iter();
        assert_eq!(Slot::parse_d8(&mut data), Ok(Slot::Data8(0x12)));
        assert_eq!(Slot::parse_d8(&mut data), Err(DecodeError::EndOfStream));
    }

    #[test]
    fn test_slot_r8() {
        assert_eq!(Slot::r8(Register8::A), Slot::Register8(Register8::A));
    }

    #[test]
    fn test_slot_r16() {
        assert_eq!(Slot::r16(Register16::BC), Slot::Register16(Register16::BC));
    }

    #[test]
    fn test_slot_addr() {
        assert_eq!(
            Slot::addr(AddrRegister::BC),
            Slot::AddrRegister(AddrRegister::BC)
        );
    }
}
