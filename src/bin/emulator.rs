use std::fmt::Display;
use std::fs::File;
use std::io::Read;

use anyhow::bail;
use anyhow::Result;
use clap::{Arg, ArgAction, Command};

extern crate gb;
use gb::decoder::Memory;
use gb::decoder::Ppu;
use gb::slots::AddrRegister;
use gb::slots::Register16;
use gb::{
    decoder::{decode, Opcode},
    slots::{Register8, Slot},
};

fn main() {
    let matches = Command::new("Emulator")
        .arg(Arg::new("bios").required(true))
        .arg(Arg::new("debug").short('d').action(ArgAction::SetTrue))
        .get_matches();
    let bios_file_name: &String = matches.get_one("bios").unwrap();
    let debug = matches.get_flag("debug");

    println!("Loading {}", bios_file_name);

    let mut bios = vec![];
    File::open(bios_file_name)
        .and_then(|mut file| file.read_to_end(&mut bios))
        .unwrap();
    run(&bios, debug).unwrap()
}

#[derive(Default)]
pub struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    //
    // Special register
    f: u8,   // Flags
    pc: u16, // program counter
    sp: u16, // program counter
}

impl Registers {
    fn flag_zero(&self) -> bool {
        (self.f & (1 << 7)) != 0
    }

    fn flag_zero_set(&mut self, value: bool) {
        self.f = if value {
            self.f | (1 << 7)
        } else {
            self.f & (!(1 << 7))
        }
    }
}

impl Display for Registers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "a: {:2x} b: {:2x} c: {:2x} d: {:2x} e: {:2x} h: {:2x} l: {:2x}",
            self.a, self.b, self.c, self.d, self.e, self.h, self.l
        )?;
        write!(f, "f: {:2x} pc: {:4x} sp: {:4x}", self.f, self.pc, self.sp)
    }
}

impl Registers {
    fn af(&self) -> u16 {
        (self.a as u16) << 8 | self.f as u16
    }

    fn bc(&self) -> u16 {
        (self.b as u16) << 8 | self.c as u16
    }

    fn de(&self) -> u16 {
        (self.d as u16) << 8 | self.e as u16
    }

    fn hl(&self) -> u16 {
        (self.h as u16) << 8 | self.l as u16
    }

    fn set_hl(&mut self, val: u16) {
        self.h = (val >> 8) as u8;
        self.l = (val & 0xFF) as u8;
    }
}

fn run(bios: &[u8], _debug: bool) -> Result<()> {
    let mut memory = Memory::from_raw(bios)?;
    let mut registers = Registers::default();
    let _ppu = Ppu::new(&memory);
    let mut clock = 0;

    loop {
        let code = decode(&memory, &mut registers.pc)?;
        println!("Code: {}", code);
        execute(code, &mut registers, &mut memory, &mut clock)?;
        println!("Registers: {}", registers);
        println!();
    }
}

/// It will fetch the value in slot and return it as a u16. This makes it compatible
/// with both 8 and 16-bits opcodes
fn fetch_value(slot: Slot, registers: &Registers, memory: &Memory) -> u16 {
    match slot {
        Slot::Register8(register) => fetch_register8(register, registers) as u16,
        Slot::AddrRegister(_addr_register) => todo!(),
        Slot::Register16(register) => fetch_register16(register, registers),
        Slot::Addr8(index) => memory.get((index + 0xFF) as u16) as u16,
        Slot::Addr16(_) => todo!(),
        Slot::Data8(value) => value as u16,
        Slot::Data16(value) => value,
    }
}

fn fetch_register8(register: Register8, registers: &Registers) -> u8 {
    match register {
        Register8::A => registers.a,
        Register8::B => registers.b,
        Register8::C => registers.c,
        Register8::D => registers.d,
        Register8::E => registers.e,
        Register8::H => registers.h,
        Register8::L => registers.l,
        Register8::F => registers.f,
    }
}

fn fetch_register16(register: Register16, registers: &Registers) -> u16 {
    match register {
        gb::slots::Register16::AF => registers.af(),
        gb::slots::Register16::BC => registers.bc(),
        gb::slots::Register16::DE => registers.de(),
        gb::slots::Register16::HL => registers.hl(),
        gb::slots::Register16::SP => registers.sp,
    }
}

fn set_value(slot: Slot, registers: &mut Registers, memory: &mut Memory, value: u16) -> Result<()> {
    match slot {
        Slot::Register8(register) => match register {
            Register8::A => registers.a = value as u8,
            Register8::B => registers.b = value as u8,
            Register8::C => registers.c = value as u8,
            Register8::D => registers.d = value as u8,
            Register8::E => registers.e = value as u8,
            Register8::H => registers.h = value as u8,
            Register8::L => registers.l = value as u8,
            Register8::F => bail!("Cannot set value to register F"),
        },
        Slot::AddrRegister(addr_register) => {
            match addr_register {
                AddrRegister::BC => {
                    memory.set(registers.bc(), value as u8);
                }
                AddrRegister::DE => {
                    memory.set(registers.de(), value as u8);
                }
                AddrRegister::HL => {
                    memory.set(registers.hl(), value as u8);
                }
                AddrRegister::C => {
                    memory.set(registers.c as u16 + 0xFF00, value as u8);
                }
            };
        }
        Slot::Register16(register) => match register {
            gb::slots::Register16::AF => bail!("Cannot set value to register AF"),
            gb::slots::Register16::BC => {
                registers.b = (value >> 8) as u8;
                registers.c = value as u8
            }
            gb::slots::Register16::DE => {
                registers.d = (value >> 8) as u8;
                registers.e = value as u8
            }
            gb::slots::Register16::HL => {
                registers.h = (value >> 8) as u8;
                registers.l = value as u8
            }
            gb::slots::Register16::SP => registers.sp = value,
        },
        Slot::Addr8(index) => memory.set(0xff00 + index as u16, value as u8),
        Slot::Addr16(_) => todo!(),
        Slot::Data8(_) => todo!(),
        Slot::Data16(_) => todo!(),
    };
    Ok(())
}

fn execute(
    code: Opcode,
    registers: &mut Registers,
    memory: &mut Memory,
    clock: &mut usize,
) -> Result<()> {
    // https://www.devrs.com/gb/files/opcodes.html
    match code {
        Opcode::Xor(from) => {
            let value = fetch_register8(Register8::A, registers) ^ fetch_register8(from, registers);
            set_value(Slot::Register8(Register8::A), registers, memory, value as u16)?;
            *clock += 4; // TODO: It's complicated
        }
        Opcode::Ld(to, from) => {
            if matches!(from, Slot::Register8(Register8::F)) {
                bail!("Invalid 'from' register F in Ld");
            }
            if matches!(from, Slot::Register16(Register16::AF)) {
                bail!("Invalid 'from' register AF in Ld");
            }
            if matches!(to, Slot::Register8(Register8::F)) {
                bail!("Invalid 'to' register F in Ld");
            }
            if matches!(to, Slot::Register16(Register16::AF)) {
                bail!("Invalid 'to' register AF in Ld");
            }
            let value = fetch_value(from, registers, memory);
            set_value(to, registers, memory, value)?;
        }
        Opcode::LdDec(from) => {
            let value = fetch_value(Slot::Register8(from), registers, memory);
            set_value(
                Slot::AddrRegister(AddrRegister::HL),
                registers,
                memory,
                value,
            )?;
            registers.set_hl(registers.hl() - 1);
        }
        Opcode::LdInc(from) => {
            let value = fetch_value(Slot::Register8(from), registers, memory);
            set_value(
                Slot::AddrRegister(AddrRegister::HL),
                registers,
                memory,
                value,
            )?;
            registers.set_hl(registers.hl() + 1);
        }
        Opcode::ComplBit(bit_index, register) => {
            let value = fetch_register8(register, registers);
            let bit_value = 1 & (value >> bit_index);
            registers.flag_zero_set(bit_value == 0); // Complementary of bit_value
        }
        Opcode::JumpRNZMemOffset(offset) => {
            if !registers.flag_zero() {
                registers.pc = ((registers.pc as i32) + (offset as i32)) as u16;
            }
        }
        _ => bail!("Unknown Opcode {}", code),
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    use gb::{
        decoder::{Memory, Opcode},
        slots::Register8,
    };

    use crate::Registers;

    use super::execute;

    #[test]
    fn test_flagzero() {
        let mut registers = Registers::default();
        registers.f = 0b10101010;
        assert!(registers.flag_zero());
        registers.flag_zero_set(false);
        assert!(!registers.flag_zero());
        assert_eq!(registers.f, 0b00101010);
        registers.flag_zero_set(true);
        assert!(registers.flag_zero());
        assert_eq!(registers.f, 0b10101010);
    }

    #[test]
    fn test_xor() {
        let mut memory = Memory::from_raw(&[0; 1]).unwrap();
        let mut registers = Registers::default();
        registers.a = 0b10100101;
        execute(
            Opcode::Xor(Register8::A),
            &mut registers,
            &mut memory,
            &mut 0,
        )
        .unwrap();
        assert_eq!(registers.a, 0);

        registers.a = 0b10100100;
        registers.b = 0b10100001;
        execute(
            Opcode::Xor(Register8::B),
            &mut registers,
            &mut memory,
            &mut 0,
        )
        .unwrap();
        assert_eq!(registers.a, 0b101);
        assert_eq!(registers.b, 0b10100001);
    }

    #[test]
    fn test_complbit() {
        let mut memory = Memory::from_raw(&[0; 1]).unwrap();
        let mut registers = Registers::default();
        registers.h = 0b10000000;
        execute(
            Opcode::ComplBit(7, Register8::H),
            &mut registers,
            &mut memory,
            &mut 0,
        )
        .unwrap();
        assert_eq!(registers.flag_zero(), false);
        execute(
            Opcode::ComplBit(6, Register8::H),
            &mut registers,
            &mut memory,
            &mut 0,
        )
        .unwrap();
        assert_eq!(registers.flag_zero(), true);

        registers.d = 0x01;
        execute(
            Opcode::ComplBit(0, Register8::D),
            &mut registers,
            &mut memory,
            &mut 0,
        )
        .unwrap();
        assert_eq!(registers.flag_zero(), false);
        execute(
            Opcode::ComplBit(2, Register8::D),
            &mut registers,
            &mut memory,
            &mut 0,
        )
        .unwrap();
        assert_eq!(registers.flag_zero(), true);
    }
}
