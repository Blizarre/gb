use crate::slots::{AddrRegister, Register16, Register8, Slot};
use crate::slots::{Register16::*, Register8::*};
use std::{
    error::Error,
    fmt::{Debug, Display},
};

#[derive(Debug, PartialEq)]
pub enum Opcode {
    Nop,
    Halt,
    Ret,
    Ld(Slot, Slot),
    Call(Slot),
    Inc(Slot),
    Cp(Slot, Slot),
    Dec(Slot),
    Sub(Slot),
    LdDec(Register8),
    LdInc(Register8),
    RotLeft(Register8),
    Push(Register16),
    Pop(Register16),
    Xor(Register8),
    ComplBit(u8, Register8),
    Jump(i8),
    JumpRZMemOffset(i8),
    JumpRNZMemOffset(i8),
}

impl Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Opcode::Cp(to, from) => write!(f, "CP {:?} {:?}", to, from),
            Opcode::Dec(from) => write!(f, "DEC {:?}", from),
            Opcode::Inc(from) => write!(f, "INC {:?}", from),
            Opcode::Push(from) => write!(f, "PUSH {:?}", from),
            Opcode::Pop(to) => write!(f, "POP {:?}", to),
            Opcode::Ld(to, from) => write!(f, "LD {:?} {:?}", to, from),
            Opcode::Call(slot) => write!(f, "CALL {:?}", slot),
            Opcode::LdInc(from) => write!(f, "LDI {:?}", from),
            Opcode::LdDec(from) => write!(f, "LDD {:?}", from),
            Opcode::Sub(from) => write!(f, "SUB A,{:?}", from),
            _ => write!(f, "{:?}", self),
        }
    }
}

#[derive(Debug)]
pub enum MemoryError {
    DataTooLarge,
}

impl Display for MemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryError::DataTooLarge => write!(f, "Data too large to fit in memory"),
        }
    }
}

impl Error for MemoryError {}

pub struct Memory([u8; 65536]);

/// A representation of the gameboy memory. The current implementation is naive and only
/// support trivial use cases. It is a start.
impl Memory {
    /// Load the complete gameboy memory from scratch. Only useful for testing for now.
    pub fn from_raw(data: &[u8]) -> Result<Memory, MemoryError> {
        if data.len() > 0xFFFF {
            return Err(MemoryError::DataTooLarge);
        }
        let mut content = [0; 65536];
        content[0..data.len()].copy_from_slice(data);
        Ok(Memory(content))
    }

    pub fn get(&self, pc: u16) -> u8 {
        self.0[pc as usize]
    }

    pub fn set(&mut self, pc: u16, value: u8) {
        self.0[pc as usize] = value
    }
}

fn fetch(data: &Memory, pc: &mut u16) -> u8 {
    let value = data.get(*pc);
    *pc += 1;
    value
}

pub fn decode(data: &Memory, pc: &mut u16) -> Result<Opcode, DecodeError> {
    let opcode = data.get(*pc);
    *pc += 1;
    // Extended Opcodes
    if opcode == 0xcb {
        return decode_extended(data.get(*pc));
    }

    if (0x40..0x80).contains(&opcode) {
        // Inside this range the arguments for the Ld Opcode
        // repeat in a specific pattern: BB, BC, BD... CB, CC, CD... AB
        // AC, AD, ...until AA. The first 3 bits represent the destination
        // and the last 3 represent the source.

        // Ld (HL), (HL) is a specific case replaced by Halt
        if opcode == 0x76 {
            return Ok(Opcode::Halt);
        }

        let address = (opcode - 0x40) as usize;
        let mapping = [
            Slot::r8(B),
            Slot::r8(C),
            Slot::r8(D),
            Slot::r8(E),
            Slot::r8(H),
            Slot::r8(L),
            Slot::AddrRegister(AddrRegister::HL),
            Slot::r8(A),
        ];
        return Ok(Opcode::Ld(mapping[address >> 3], mapping[address & 0x7]));
    }
    Ok(match opcode {
        0x00 => Opcode::Nop,
        0x01 => Opcode::Ld(Slot::r16(BC), Slot::parse_d16(data, pc)),
        0x02 => Opcode::Ld(Slot::addr(AddrRegister::BC), Slot::r8(A)),
        0x03 => Opcode::Inc(Slot::r16(BC)),
        0x04 => Opcode::Inc(Slot::r8(B)),
        0x05 => Opcode::Dec(Slot::r8(B)),
        0x06 => Opcode::Ld(Slot::r8(B), Slot::parse_d8(data, pc)),
        0x0c => Opcode::Inc(Slot::r8(C)),
        0x0d => Opcode::Dec(Slot::r8(C)),
        0x0e => Opcode::Ld(Slot::r8(C), Slot::parse_d8(data, pc)),
        0x11 => Opcode::Ld(Slot::r16(DE), Slot::parse_d16(data, pc)),
        0x13 => Opcode::Inc(Slot::r16(DE)),
        0x14 => Opcode::Inc(Slot::r8(D)),
        0x15 => Opcode::Dec(Slot::r8(D)),
        0x16 => Opcode::Ld(Slot::r8(D), Slot::parse_d8(data, pc)),
        0x17 => Opcode::RotLeft(A),
        0x18 => Opcode::Jump(fetch(data, pc) as i8),
        0x1a => Opcode::Ld(Slot::r8(A), Slot::addr(AddrRegister::DE)),
        0x1b => Opcode::Dec(Slot::r16(DE)),
        0x1c => Opcode::Inc(Slot::r8(E)),
        0x1d => Opcode::Dec(Slot::r8(E)),
        0x1e => Opcode::Ld(Slot::r8(E), Slot::parse_d8(data, pc)),
        0x20 => Opcode::JumpRNZMemOffset(fetch(data, pc) as i8),
        0x21 => Opcode::Ld(Slot::r16(HL), Slot::parse_d16(data, pc)),
        0x22 => Opcode::LdInc(A),
        0x23 => Opcode::Inc(Slot::r16(HL)),
        0x24 => Opcode::Inc(Slot::r8(H)),
        0x25 => Opcode::Dec(Slot::r8(H)),
        0x28 => Opcode::JumpRZMemOffset(fetch(data, pc) as i8),
        0x2e => Opcode::Ld(Slot::r8(L), Slot::parse_d8(data, pc)),
        0x31 => Opcode::Ld(Slot::r16(SP), Slot::parse_d16(data, pc)),
        0x32 => Opcode::LdDec(A),
        0x34 => Opcode::Inc(Slot::AddrRegister(AddrRegister::HL)),
        0x35 => Opcode::Dec(Slot::AddrRegister(AddrRegister::HL)),
        0x3d => Opcode::Dec(Slot::r8(A)),
        0x3e => Opcode::Ld(Slot::r8(A), Slot::parse_d8(data, pc)),
        0x90 => Opcode::Sub(Slot::r8(B)),
        0x91 => Opcode::Sub(Slot::r8(C)),
        0x92 => Opcode::Sub(Slot::r8(D)),
        0x93 => Opcode::Sub(Slot::r8(E)),
        0x94 => Opcode::Sub(Slot::r8(H)),
        0x95 => Opcode::Sub(Slot::r8(L)),
        0x96 => Opcode::Sub(Slot::AddrRegister(AddrRegister::HL)),
        0x97 => Opcode::Sub(Slot::r8(A)),
        0xaf => Opcode::Xor(A),
        0xc1 => Opcode::Pop(BC),
        0xc5 => Opcode::Push(BC),
        0xc9 => Opcode::Ret,
        0xcd => Opcode::Call(Slot::parse_d16(data, pc)),
        0xe0 => Opcode::Ld(Slot::parse_a8(data, pc), Slot::r8(A)),
        0xe2 => Opcode::Ld(Slot::addr(AddrRegister::C), Slot::r8(A)),
        0xea => Opcode::Ld(Slot::parse_a16(data, pc), Slot::r8(A)),
        0xf0 => Opcode::Ld(Slot::r8(A), Slot::parse_a8(data, pc)),
        0xf1 => Opcode::Pop(AF),
        0xfe => Opcode::Cp(Slot::r8(A), Slot::parse_d8(data, pc)),
        _ => return Err(DecodeError::UnknownOpcode(opcode)),
    })
}

#[derive(PartialEq)]
pub enum DecodeError {
    UnknownOpcode(u8),
    UnknownExtendedOpcode(u8),
}

impl Debug for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <DecodeError as Display>::fmt(self, f)
    }
}
impl Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownOpcode(opcode) => write!(f, "Unknown Opcode 0x{:x}", opcode),
            Self::UnknownExtendedOpcode(opcode) => {
                write!(f, "Unknown Extended opcode 0x{:x}", opcode)
            }
        }
    }
}

impl Error for DecodeError {}

fn decode_extended(data: u8) -> Result<Opcode, DecodeError> {
    Ok(match data {
        0x11 => Opcode::RotLeft(C),
        0x7c => Opcode::ComplBit(7, H),
        0x4f => Opcode::ComplBit(1, A),
        _ => return Err(DecodeError::UnknownExtendedOpcode(data)),
    })
}

#[cfg(test)]
mod test {
    use super::decode;
    use super::{Memory, Opcode, Register8::*};
    use crate::{slots::AddrRegister, slots::Slot};

    #[test]
    fn decode_ld_band() {
        let mut pc = 0;
        let memory = Memory::from_raw(&[0x40u8, 0x5fu8, 0x66u8, 0x68u8]).unwrap();
        assert_eq!(
            decode(&memory, &mut pc).unwrap(),
            Opcode::Ld(Slot::Register8(B), Slot::Register8(B))
        );
        assert_eq!(
            decode(&memory, &mut pc).unwrap(),
            Opcode::Ld(Slot::Register8(E), Slot::Register8(A))
        );
        assert_eq!(
            decode(&memory, &mut pc).unwrap(),
            Opcode::Ld(Slot::Register8(H), Slot::AddrRegister(AddrRegister::HL),)
        );
        assert_eq!(
            decode(&memory, &mut pc).unwrap(),
            Opcode::Ld(Slot::Register8(L), Slot::Register8(B)),
        );

        let mut pc = 0;
        let memory = Memory::from_raw(&[0x7du8, 0x76u8]).unwrap();

        assert_eq!(
            decode(&memory, &mut pc).unwrap(),
            Opcode::Ld(Slot::Register8(A), Slot::Register8(L)),
        );
        assert_eq!(decode(&memory, &mut pc).unwrap(), Opcode::Halt);

        assert_eq!(pc, 2);
    }
}
