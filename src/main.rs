use std::collections::BTreeMap;
use std::fmt::{Debug, Display};
use std::{error::Error, fs::File, io::Read};

use clap::{Arg, ArgAction, Command};
extern crate clap;

mod slots;
use slots::{AddrRegister, Register16, Register16::*, Register8, Register8::*, Slot};

use annotations::{Annotation, Purpose};

mod annotations;

fn main() {
    let matches = Command::new("Disassembler")
        .arg(Arg::new("file").required(true))
        .arg(Arg::new("annotation").required(true))
        .arg(Arg::new("debug").short('d').action(ArgAction::SetTrue))
        .get_matches();
    let file_name: &String = matches.get_one("file").unwrap();
    let file_name_annotation: &String = matches.get_one("annotation").unwrap();

    let annotations =
        Annotation::parse(file_name_annotation).expect("Error loading the annotation file");

    println!("{}", file_name);
    disassemble(
        &mut File::open(file_name).unwrap(),
        annotations,
        matches.get_flag("debug"),
    )
    .unwrap();
}

fn disassemble(
    file: &mut File,
    annotations: BTreeMap<usize, Vec<Annotation>>,
    debug: bool,
) -> Result<(), Box<dyn Error + 'static>> {
    let empty_vec = vec![];
    let mut buf = vec![];
    file.read_to_end(&mut buf)?;
    let mut idx: usize = 0;

    while idx < buf.len() {
        let mut comment = String::new();
        let mut goto = String::new();
        let mut label = String::new();
        let mut skip = 0;
        let annotations = annotations.get(&idx).unwrap_or(&empty_vec);

        for annotation in annotations {
            match annotation.purpose {
                Purpose::Comment => comment = format!(" ; {}", &annotation.value),
                Purpose::Goto => goto = format!("-> {}", &annotation.value),
                Purpose::Label => label = format!(":{}", &annotation.value),
                Purpose::Section => {
                    println!("-- {} --", annotation.value)
                }
                Purpose::Data => {
                    skip = usize::from_str_radix(annotation.value.trim_start_matches("0x"), 16)
                        .unwrap();
                }
            }
        }

        if skip > 0 {
            println!(
                "0x{:04x}-0x{:04x} {} {} {}",
                idx,
                idx + skip - 1,
                label,
                goto,
                comment
            );
            idx += skip;
        } else {
            let (len, opcode) = decode(&buf[idx..]);
            if debug {
                print!("{:02x} ", buf[idx]);
            }
            println!("0x{:04x} {} {} {} {}", idx, opcode, label, goto, comment);
            idx += len;
        }
    }
    Ok(())
}

#[derive(Debug)]
enum Opcode {
    Nop,
    Ld(Slot, Slot),
    Call(Slot),
    Ld8(Register8, Register8),
    Inc8(Register8),
    Dec8(Register8),
    LdToMemDec(Register16, Register8),
    RotLeft(Register8),
    Inc16(Register16),
    Push(Register16),
    Pop(Register16),
    Xor(Register8, Register8),
    ComplBit(u8, Register8),
    JumpNZMemOffset(i8),
}

impl Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Opcode::Ld(to, from) => write!(f, "LD {:?} {:?}", to, from),
            Opcode::Call(slot) => write!(f, "CALL {:?}", slot),
            _ => write!(f, "{:?}", self),
        }
    }
}

fn decode(data: &[u8]) -> (usize, Opcode) {
    // Extended Opcodes
    if data[0] == 0xcb {
        return decode_extended(&data[1..]);
    }
    match data[0] {
        0x00 => (1, Opcode::Nop),
        0x01 => (3, Opcode::Ld(Slot::r16(BC), Slot::parse_d16(&data[1..]))),
        0x02 => (
            1,
            Opcode::Ld(Slot::AddrRegister(AddrRegister::BC), Slot::r8(A)),
        ),
        0x03 => (1, Opcode::Inc16(Register16::BC)),
        0x04 => (1, Opcode::Inc8(Register8::B)),
        0x05 => (1, Opcode::Dec8(Register8::B)),
        0x06 => (2, Opcode::Ld(Slot::r8(B), Slot::Data8(data[1]))),
        0x0c => (1, Opcode::Inc8(Register8::C)),
        0x0e => (2, Opcode::Ld(Slot::r8(C), Slot::Addr8(data[1]))),
        0x11 => (3, Opcode::Ld(Slot::r16(DE), Slot::parse_d16(&data[1..]))),
        0x1a => (
            1,
            Opcode::Ld(Slot::r8(A), Slot::AddrRegister(AddrRegister::DE)),
        ),
        0x17 => (1, Opcode::RotLeft(Register8::A)),
        0x20 => (2, Opcode::JumpNZMemOffset(data[1] as i8)),
        0x21 => (3, Opcode::Ld(Slot::r16(HL), Slot::parse_d16(&data[1..]))),
        0x31 => (3, Opcode::Ld(Slot::r16(SP), Slot::parse_d16(&data[1..]))),
        0x32 => (1, Opcode::LdToMemDec(Register16::HL, Register8::A)),
        0x3e => (2, Opcode::Ld(Slot::r8(A), Slot::parse_d8(&data[1..]))),
        0x45 => (1, Opcode::Ld(Slot::r8(B), Slot::r8(L))),
        0x46 => (
            1,
            Opcode::Ld(Slot::r8(B), Slot::AddrRegister(AddrRegister::HL)),
        ),
        0x4c => (1, Opcode::Ld(Slot::r8(H), Slot::r8(C))),
        0x4f => (1, Opcode::Ld(Slot::r8(C), Slot::r8(A))),
        0x77 => (
            1,
            Opcode::Ld(Slot::AddrRegister(AddrRegister::HL), Slot::r8(A)),
        ),
        0x7f => (1, Opcode::Ld8(Register8::A, Register8::A)),
        0xaf => (1, Opcode::Xor(Register8::A, Register8::A)),
        0xc1 => (1, Opcode::Pop(Register16::BC)),
        0xc5 => (1, Opcode::Push(Register16::BC)),
        0xcd => (3, Opcode::Call(Slot::parse_d16(&data[1..]))),
        0xe0 => (2, Opcode::Ld(Slot::parse_a8(&data[1..]), Slot::r8(A))),
        0xe2 => (
            1,
            Opcode::Ld(Slot::AddrRegister(AddrRegister::C), Slot::r8(A)),
        ),
        _ => panic!("Unknown Opcode {:x}", data[0]),
    }
}

fn decode_extended(data: &[u8]) -> (usize, Opcode) {
    match data[0] {
        0x11 => (2, Opcode::RotLeft(Register8::C)),
        0x7c => (2, Opcode::ComplBit(7, Register8::H)),
        0x4f => (2, Opcode::ComplBit(1, Register8::A)),
        _ => panic!("Unknown CB Opcode {:x}", data[0]),
    }
}
