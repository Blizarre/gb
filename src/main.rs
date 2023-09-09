use std::collections::BTreeMap;
use std::fmt::{Debug, Display};
use std::{error::Error, fs::File, io::Read};

use clap::{Arg, ArgAction, Command};
extern crate clap;

mod slots;
use slots::{AddrRegister, Register16, Register16::*, Register8, Register8::*, Slot};

use annotations::{Annotation, Purpose};

mod annotations;

struct IndexedIter<T> {
    it: std::vec::IntoIter<T>,
    index: usize,
}

impl<T> Iterator for IndexedIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        self.it.next()
    }
}

impl<T> IndexedIter<T> {
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn from_vec(vec: Vec<T>) -> Self {
        Self {
            it: vec.into_iter(),
            index: 0,
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
        Annotation::parse_file(file_name_annotation).expect("Error loading the annotation file");

    println!("{}", file_name);

    let mut buf = vec![];
    File::open(file_name)
        .and_then(|mut file| file.read_to_end(&mut buf))
        .unwrap();
    disassemble(buf, annotations, matches.get_flag("debug")).unwrap()
}

fn disassemble(
    data: Vec<u8>,
    annotations: BTreeMap<usize, Vec<Annotation>>,
    debug: bool,
) -> Result<(), Box<dyn Error + 'static>> {
    let empty_vec = vec![];
    let mut it = IndexedIter::from_vec(data.clone());

    loop {
        let mut comment = String::new();
        let mut goto = String::new();
        let mut label = String::new();
        let mut skip = 0;
        let annotations = annotations.get(&it.index()).unwrap_or(&empty_vec);

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
                it.index(),
                it.index() + skip - 1,
                label,
                goto,
                comment
            );
            it.nth(skip - 1);
        } else {
            let current_index = it.index();
            let opcode = decode(&mut it).unwrap();
            if debug {
                print!("{:02x} ", data[current_index]);
            }
            println!(
                "0x{:04x} {} {} {} {}",
                current_index, opcode, label, goto, comment
            );
        }
    }
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
    LdToMemInc(Register16, Register8),
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

fn decode(data: &mut impl Iterator<Item = u8>) -> Result<Opcode, DecodeError> {
    let opcode = data.next().ok_or(DecodeError::EndOfStream)?;
    // Extended Opcodes
    if opcode == 0xcb {
        return decode_extended(data.next().ok_or(DecodeError::EndOfStream)?);
    }
    Ok(match opcode {
        0x00 => Opcode::Nop,
        0x01 => Opcode::Ld(Slot::r16(BC), Slot::parse_d16(data)?),
        0x02 => Opcode::Ld(Slot::addr(AddrRegister::BC), Slot::r8(A)),
        0x03 => Opcode::Inc16(Register16::BC),
        0x04 => Opcode::Inc8(Register8::B),
        0x05 => Opcode::Dec8(Register8::B),
        0x06 => Opcode::Ld(Slot::r8(B), Slot::parse_d8(data)?),
        0x0c => Opcode::Inc8(Register8::C),
        0x0e => Opcode::Ld(Slot::r8(C), Slot::parse_a8(data)?),
        0x11 => Opcode::Ld(Slot::r16(DE), Slot::parse_d16(data)?),
        0x1a => Opcode::Ld(Slot::r8(A), Slot::addr(AddrRegister::DE)),
        0x17 => Opcode::RotLeft(Register8::A),
        0x20 => Opcode::JumpNZMemOffset(data.next().ok_or(DecodeError::EndOfStream)? as i8),
        0x21 => Opcode::Ld(Slot::r16(HL), Slot::parse_d16(data)?),
        0x22 => Opcode::LdToMemInc(Register16::HL, Register8::A),
        0x31 => Opcode::Ld(Slot::r16(SP), Slot::parse_d16(data)?),
        0x32 => Opcode::LdToMemDec(Register16::HL, Register8::A),
        0x3e => Opcode::Ld(Slot::r8(A), Slot::parse_d8(data)?),
        0x45 => Opcode::Ld(Slot::r8(B), Slot::r8(L)),
        0x46 => Opcode::Ld(Slot::r8(B), Slot::addr(AddrRegister::HL)),
        0x4c => Opcode::Ld(Slot::r8(H), Slot::r8(C)),
        0x4f => Opcode::Ld(Slot::r8(C), Slot::r8(A)),
        0x77 => Opcode::Ld(Slot::addr(AddrRegister::HL), Slot::r8(A)),
        0x7f => Opcode::Ld8(Register8::A, Register8::A),
        0xaf => Opcode::Xor(Register8::A, Register8::A),
        0xc1 => Opcode::Pop(Register16::BC),
        0xc5 => Opcode::Push(Register16::BC),
        0xcd => Opcode::Call(Slot::parse_d16(data)?),
        0xe0 => Opcode::Ld(Slot::parse_a8(data)?, Slot::r8(A)),
        0xe2 => Opcode::Ld(Slot::addr(AddrRegister::C), Slot::r8(A)),
        _ => return Err(DecodeError::UnknownOpcode(opcode)),
    })
}

#[derive(PartialEq)]
pub enum DecodeError {
    EndOfStream,
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
            Self::EndOfStream => write!(f, "End of stream detected during opcode decoding"),
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
        0x11 => Opcode::RotLeft(Register8::C),
        0x7c => Opcode::ComplBit(7, Register8::H),
        0x4f => Opcode::ComplBit(1, Register8::A),
        _ => return Err(DecodeError::UnknownExtendedOpcode(data)),
    })
}
