use std::collections::BTreeMap;
use std::fmt::{Debug, Display};
use std::{error::Error, fs::File, io::Read};

use clap::{Arg, ArgAction, Command};
use itertools::Itertools;
extern crate clap;

#[derive(PartialEq)]
enum Purpose {
    Comment,
    Section,
    Goto,
    Label,
    Data,
}

impl From<&str> for Purpose {
    fn from(mnemonic: &str) -> Self {
        match mnemonic {
            "C" => Purpose::Comment,
            "S" => Purpose::Section,
            "G" => Purpose::Goto,
            "L" => Purpose::Label,
            "D" => Purpose::Data,
            _ => panic!("Unknown pattern"),
        }
    }
}

struct Annotation {
    location: usize,
    purpose: Purpose,
    value: String,
}

impl From<&str> for Annotation {
    fn from(line: &str) -> Self {
        let items: Vec<&str> = line.splitn(3, ' ').collect();
        Annotation {
            location: usize::from_str_radix(items[0].trim_start_matches("0x"), 16).unwrap(),
            purpose: Purpose::from(items[1]),
            value: items[2].to_string(),
        }
    }
}
fn main() {
    let matches = Command::new("Disassembler")
        .arg(Arg::new("file").required(true))
        .arg(Arg::new("annotation").required(true))
        .arg(Arg::new("debug").short('d').action(ArgAction::SetTrue))
        .get_matches();
    let file_name: &String = matches.get_one("file").unwrap();
    let file_name_annotation: &String = matches.get_one("annotation").unwrap();

    let annotations =
        load_annotations(file_name_annotation).expect("Error loading the annotation file");

    println!("{}", file_name);
    disassemble(
        &mut File::open(file_name).unwrap(),
        annotations,
        matches.get_flag("debug"),
    )
    .unwrap();
}

fn load_annotations(
    file_name_annotation: &String,
) -> Result<BTreeMap<usize, Vec<Annotation>>, std::io::Error> {
    let mut tmp = String::new();
    File::open(file_name_annotation)
        .map(|mut f| f.read_to_string(&mut tmp))
        .map(|_| {
            tmp.split('\n')
                .filter(|l| !l.trim().is_empty())
                .map(Annotation::from)
                .sorted_by_key(|a| a.location)
                .group_by(|a| a.location)
                .into_iter()
                .map(|(key, group)| (key, group.collect()))
                .collect::<BTreeMap<usize, Vec<Annotation>>>()
        })
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

#[allow(dead_code)]
#[derive(Debug)]
enum Register16 {
    BC,
    DE,
    FG,
    HL,
    SP,
}

#[allow(dead_code)]
#[derive(Debug)]
enum Register8 {
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

#[derive(Debug)]
enum Opcode {
    Nop,
    LdImm16(Register16, u16),
    CallMem(u16),
    Ld8(Register8, Register8),
    Inc8(Register8),
    Dec8(Register8),
    LdFromMem(Register8, Register16),
    LdToMem8FromReg(Register8, Register8),
    LdToMem8Imm(u8, Register8),
    LdToMem(Register16, Register8),
    LdToMemDec(Register16, Register8),
    RotLeft(Register8),
    LdImm8(Register8, u8),
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
            Opcode::Ld8(to, from) => write!(f, "LD {:?} {:?}", to, from),
            Opcode::CallMem(u16) => write!(f, "CALL u16 0x{:x}", u16),
            Opcode::LdImm16(r16, u16) => write!(f, "LD {:?} 0x{:x}", r16, u16),
            Opcode::LdImm8(r8, u8) => write!(f, "LD {:?} 0x{:x}", r8, u8),
            Opcode::LdToMem8Imm(u8, r8) => write!(f, "LD (0x{:x}) {:?}", u8, r8),
            _ => write!(f, "{:?}", self),
        }
    }
}

fn decode_u16(data: &[u8]) -> u16 {
    u16::from_le_bytes(
        data[..2]
            .try_into()
            .expect("Enf of file in the middle of a constant"),
    )
}

fn decode(data: &[u8]) -> (usize, Opcode) {
    // Extended Opcodes
    if data[0] == 0xcb {
        return decode_extended(&data[1..]);
    }
    match data[0] {
        0x00 => (1, Opcode::Nop),
        0x01 => (3, Opcode::LdImm16(Register16::BC, decode_u16(&data[1..]))),
        0x02 => (1, Opcode::LdToMem(Register16::BC, Register8::A)),
        0x03 => (1, Opcode::Inc16(Register16::BC)),
        0x04 => (1, Opcode::Inc8(Register8::B)),
        0x05 => (1, Opcode::Dec8(Register8::B)),
        0x06 => (2, Opcode::LdImm8(Register8::B, data[0])),
        0x0c => (1, Opcode::Inc8(Register8::C)),
        0x0e => (2, Opcode::LdImm8(Register8::C, data[1])),
        0x11 => (3, Opcode::LdImm16(Register16::DE, decode_u16(&data[1..]))),
        0x1a => (1, Opcode::LdFromMem(Register8::A, Register16::DE)),
        0x17 => (1, Opcode::RotLeft(Register8::A)),
        0x20 => (2, Opcode::JumpNZMemOffset(data[1] as i8)),
        0x21 => (3, Opcode::LdImm16(Register16::HL, decode_u16(&data[1..]))),
        0x31 => (3, Opcode::LdImm16(Register16::SP, decode_u16(&data[1..]))),
        0x32 => (1, Opcode::LdToMemDec(Register16::HL, Register8::A)),
        0x3e => (2, Opcode::LdImm8(Register8::A, data[1])),
        0x45 => (1, Opcode::Ld8(Register8::B, Register8::L)),
        0x46 => (1, Opcode::LdFromMem(Register8::B, Register16::HL)),
        0x4c => (1, Opcode::Ld8(Register8::H, Register8::C)),
        0x4f => (1, Opcode::Ld8(Register8::C, Register8::A)),
        0x77 => (1, Opcode::LdToMem(Register16::HL, Register8::A)),
        0x7f => (1, Opcode::Ld8(Register8::A, Register8::A)),
        0xaf => (1, Opcode::Xor(Register8::A, Register8::A)),
        0xc1 => (1, Opcode::Pop(Register16::BC)),
        0xc5 => (1, Opcode::Push(Register16::BC)),
        0xcd => (3, Opcode::CallMem(decode_u16(&data[1..]))),
        0xe0 => (2, Opcode::LdToMem8Imm(data[1], Register8::A)),
        0xe2 => (1, Opcode::LdToMem8FromReg(Register8::C, Register8::A)),
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
