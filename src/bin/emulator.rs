use std::fmt::Display;
use std::fs::File;
use std::io::Read;

use anyhow::bail;
use anyhow::Result;
use clap::{Arg, ArgAction, Command};

extern crate gb;
use gb::decoder::Memory;
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
    fn _af(&self) -> u16 {
        (self.a as u16) << 8 & self.f as u16
    }

    fn bc(&self) -> u16 {
        (self.b as u16) << 8 & self.c as u16
    }

    fn de(&self) -> u16 {
        (self.d as u16) << 8 & self.e as u16
    }

    fn hl(&self) -> u16 {
        (self.h as u16) << 8 & self.l as u16
    }
}

fn run(bios: &[u8], _debug: bool) -> Result<()> {
    let mut memory = Memory::from_raw(bios)?;
    let mut registers = Registers::default();

    let code = decode(&memory, &mut registers.pc)?;
    println!("Code: {}", code);

    execute(code, &mut registers, &mut memory)?;

    println!("Registers: {}", registers);

    Ok(())
}

fn execute(code: Opcode, registers: &mut Registers, _memory: &mut Memory) -> Result<()> {
    match code {
        Opcode::Ld(to, from) => {
            let value = match from {
                Slot::Register8(register) => {
                    (match register {
                        Register8::A => registers.a,
                        Register8::B => registers.b,
                        Register8::C => registers.c,
                        Register8::D => registers.d,
                        Register8::E => registers.e,
                        Register8::H => registers.h,
                        Register8::L => registers.l,
                        Register8::F => bail!("Invalid register F in Ld"),
                    }) as u16
                }
                Slot::AddrRegister(_addr_register) => todo!(),
                Slot::Register16(register) => match register {
                    gb::slots::Register16::AF => bail!("Invalid register AF in Ld"),
                    gb::slots::Register16::BC => registers.bc(),
                    gb::slots::Register16::DE => registers.de(),
                    gb::slots::Register16::HL => registers.hl(),
                    gb::slots::Register16::SP => registers.sp,
                },
                Slot::Addr8(_) => todo!(),
                Slot::Addr16(_) => todo!(),
                Slot::Data8(value) => value as u16,
                Slot::Data16(value) => value,
            };
            match to {
                Slot::Register8(register) => match register {
                    Register8::A => registers.a = value as u8,
                    Register8::B => registers.b = value as u8,
                    Register8::C => registers.c = value as u8,
                    Register8::D => registers.d = value as u8,
                    Register8::E => registers.e = value as u8,
                    Register8::H => registers.h = value as u8,
                    Register8::L => registers.l = value as u8,
                    Register8::F => bail!("Invalid register F in Ld"),
                },
                Slot::AddrRegister(_addr_register) => todo!(),
                Slot::Register16(register) => match register {
                    gb::slots::Register16::AF => bail!("Invalid register AF in Ld"),
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
                Slot::Addr8(_) => todo!(),
                Slot::Addr16(_) => todo!(),
                Slot::Data8(_) => todo!(),
                Slot::Data16(_) => todo!(),
            };
        }
        _ => {}
    };
    Ok(())
}
