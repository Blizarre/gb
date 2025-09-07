use std::fmt::Debug;

use crate::decoder::Memory;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum AddrRegister {
    BC,
    DE,
    HL,
    C,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Register16 {
    AF,
    BC,
    DE,
    HL,
    SP,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Register8 {
    A,
    B,
    C,
    D,
    E,
    F,
    L,
    H,
}

#[allow(dead_code)]
#[derive(PartialEq, Clone, Copy)]
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
    pub fn parse_a16(data: &Memory, pc: &mut u16) -> Self {
        Slot::Addr16(decode_u16(data, pc))
    }

    pub fn parse_a8(data: &Memory, pc: &mut u16) -> Self {
        Slot::Addr8(decode_u8(data, pc))
    }

    pub fn parse_d16(data: &Memory, pc: &mut u16) -> Self {
        Slot::Data16(decode_u16(data, pc))
    }

    pub fn parse_d8(data: &Memory, pc: &mut u16) -> Self {
        Slot::Data8(decode_u8(data, pc))
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

fn decode_u8(data: &Memory, pc: &mut u16) -> u8 {
    let value = data.get(*pc);
    *pc += 1;
    value
}

fn decode_u16(data: &Memory, pc: &mut u16) -> u16 {
    let value = u16::from_le_bytes([data.get(*pc), data.get(*pc + 1u16)]);
    *pc += 2;
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_memory(raw_data: &[u8]) -> Memory {
        Memory::from_raw(raw_data).unwrap()
    }

    #[test]
    fn test_slot_parse_a16() {
        let mut data = create_memory(&[0x12, 0x34]);
        let mut pc = 0;
        assert_eq!(Slot::parse_a16(&mut data, &mut pc), Slot::Addr16(0x3412));
        assert_eq!(pc, 2);
    }

    #[test]
    fn test_slot_parse_a8() {
        let mut data = create_memory(&[0x12]);
        let mut pc = 0;
        assert_eq!(Slot::parse_a8(&mut data, &mut pc), Slot::Addr8(0x12));
        assert_eq!(pc, 1);
    }

    #[test]
    fn test_slot_parse_d16() {
        let mut data = create_memory(&[0x12, 0x34]);
        let mut pc = 0;
        assert_eq!(Slot::parse_d16(&mut data, &mut pc), Slot::Data16(0x3412));
        assert_eq!(pc, 2);
    }

    #[test]
    fn test_slot_parse_d8() {
        let mut data = create_memory(&[0x12]);
        let mut pc = 0;
        assert_eq!(Slot::parse_d8(&mut data, &mut pc), Slot::Data8(0x12));
        assert_eq!(pc, 1);
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
