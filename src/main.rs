use std::collections::BTreeMap;
use std::fmt::{Debug, Display};
use std::{error::Error, fs::File, io::Read};

use clap::{Arg, ArgAction, Command};
extern crate clap;

mod slots;
use indexediter::IndexedIter;
use slots::{AddrRegister, Register16, Register16::*, Register8, Register8::*, Slot};

use annotations::{Annotation, Purpose};

mod annotations;
mod indexediter;

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
        let mut label = None;
        let mut skip = 0;
        let annotations = annotations.get(&it.index()).unwrap_or(&empty_vec);

        for annotation in annotations {
            match annotation.purpose {
                Purpose::Comment => comment = format!(" ; {}", &annotation.value),
                Purpose::Goto => goto = format!("-> {}", &annotation.value),
                Purpose::Label => label = Some(annotation.value.to_string()),
                Purpose::Section => {
                    println!("\n-- {} --", annotation.value)
                }
                Purpose::Data => {
                    skip = usize::from_str_radix(annotation.value.trim_start_matches("0x"), 16)
                        .unwrap();
                }
            }
        }

        if let Some(l) = label {
            println!("{}:", l);
        }
        if skip > 0 {
            println!(
                "Skip 0x{:04x}-0x{:04x} {} {}",
                it.index(),
                it.index() + skip - 1,
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
            // Display the destination address of a jump if it has not been provided
            goto = if goto.is_empty() {
                let fmt_offset =
                    |offset| format!("-> 0x{:x}", it.index() as isize + offset as isize);
                match opcode {
                    Opcode::Jump(offset) => fmt_offset(offset),
                    Opcode::JumpRNZMemOffset(offset) => fmt_offset(offset),
                    Opcode::JumpRZMemOffset(offset) => fmt_offset(offset),
                    _ => String::new(),
                }
            } else {
                goto
            };

            println!(
                "    0x{:04x} {} {} {}",
                current_index, opcode, goto, comment
            );
        }
    }
}

#[derive(Debug, PartialEq)]
enum Opcode {
    Nop,
    Halt,
    Ret,
    Ld(Slot, Slot),
    Call(Slot),
    Inc(Slot),
    Cp(Slot, Slot),
    Dec(Slot),
    Sub(Slot),
    LdToMemDec(Register16, Register8),
    LdToMemInc(Register16, Register8),
    RotLeft(Register8),
    Push(Register16),
    Pop(Register16),
    Xor(Register8, Register8),
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
            Opcode::LdToMemInc(to, from) => write!(f, "LD ({:?}++) {:?}", to, from),
            Opcode::LdToMemDec(to, from) => write!(f, "LD ({:?}--) {:?}", to, from),
            Opcode::Sub(from) => write!(f, "SUB A,{:?}", from),
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
        0x01 => Opcode::Ld(Slot::r16(BC), Slot::parse_d16(data)?),
        0x02 => Opcode::Ld(Slot::addr(AddrRegister::BC), Slot::r8(A)),
        0x03 => Opcode::Inc(Slot::r16(BC)),
        0x04 => Opcode::Inc(Slot::r8(B)),
        0x05 => Opcode::Dec(Slot::r8(B)),
        0x06 => Opcode::Ld(Slot::r8(B), Slot::parse_d8(data)?),
        0x0c => Opcode::Inc(Slot::r8(C)),
        0x0d => Opcode::Dec(Slot::r8(C)),
        0x0e => Opcode::Ld(Slot::r8(C), Slot::parse_d8(data)?),
        0x11 => Opcode::Ld(Slot::r16(DE), Slot::parse_d16(data)?),
        0x13 => Opcode::Inc(Slot::r16(DE)),
        0x14 => Opcode::Inc(Slot::r8(D)),
        0x15 => Opcode::Dec(Slot::r8(D)),
        0x16 => Opcode::Ld(Slot::r8(D), Slot::parse_d8(data)?),
        0x17 => Opcode::RotLeft(A),
        0x18 => Opcode::Jump(data.next().ok_or(DecodeError::EndOfStream)? as i8),
        0x1a => Opcode::Ld(Slot::r8(A), Slot::addr(AddrRegister::DE)),
        0x1b => Opcode::Dec(Slot::r16(DE)),
        0x1c => Opcode::Inc(Slot::r8(E)),
        0x1d => Opcode::Dec(Slot::r8(E)),
        0x1e => Opcode::Ld(Slot::r8(E), Slot::parse_d8(data)?),
        0x20 => Opcode::JumpRNZMemOffset(data.next().ok_or(DecodeError::EndOfStream)? as i8),
        0x21 => Opcode::Ld(Slot::r16(HL), Slot::parse_d16(data)?),
        0x22 => Opcode::LdToMemInc(HL, A),
        0x23 => Opcode::Inc(Slot::r16(HL)),
        0x24 => Opcode::Inc(Slot::r8(H)),
        0x25 => Opcode::Dec(Slot::r8(H)),
        0x28 => Opcode::JumpRZMemOffset(data.next().ok_or(DecodeError::EndOfStream)? as i8),
        0x2e => Opcode::Ld(Slot::r8(L), Slot::parse_d8(data)?),
        0x31 => Opcode::Ld(Slot::r16(SP), Slot::parse_d16(data)?),
        0x32 => Opcode::LdToMemDec(HL, A),
        0x34 => Opcode::Inc(Slot::AddrRegister(AddrRegister::HL)),
        0x35 => Opcode::Dec(Slot::AddrRegister(AddrRegister::HL)),
        0x3d => Opcode::Dec(Slot::r8(A)),
        0x3e => Opcode::Ld(Slot::r8(A), Slot::parse_d8(data)?),
        0x90 => Opcode::Sub(Slot::r8(B)),
        0x91 => Opcode::Sub(Slot::r8(C)),
        0x92 => Opcode::Sub(Slot::r8(D)),
        0x93 => Opcode::Sub(Slot::r8(E)),
        0x94 => Opcode::Sub(Slot::r8(H)),
        0x95 => Opcode::Sub(Slot::r8(L)),
        0x96 => Opcode::Sub(Slot::AddrRegister(AddrRegister::HL)),
        0x97 => Opcode::Sub(Slot::r8(A)),
        0xaf => Opcode::Xor(A, A),
        0xc1 => Opcode::Pop(BC),
        0xc5 => Opcode::Push(BC),
        0xc9 => Opcode::Ret,
        0xcd => Opcode::Call(Slot::parse_d16(data)?),
        0xe0 => Opcode::Ld(Slot::parse_a8(data)?, Slot::r8(A)),
        0xe2 => Opcode::Ld(Slot::addr(AddrRegister::C), Slot::r8(A)),
        0xea => Opcode::Ld(Slot::parse_a16(data)?, Slot::r8(A)),
        0xf0 => Opcode::Ld(Slot::r8(A), Slot::parse_a8(data)?),
        0xf1 => Opcode::Pop(AF),
        0xfe => Opcode::Cp(Slot::r8(A), Slot::parse_d8(data)?),
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
        0x11 => Opcode::RotLeft(C),
        0x7c => Opcode::ComplBit(7, H),
        0x4f => Opcode::ComplBit(1, A),
        _ => return Err(DecodeError::UnknownExtendedOpcode(data)),
    })
}

#[cfg(test)]
mod test {
    use super::decode;
    use super::{slots::AddrRegister, slots::Slot, Opcode, Register8::*};

    #[test]
    fn decode_ld_band() {
        assert_eq!(
            decode(&mut [0x40u8].iter().copied()).unwrap(),
            Opcode::Ld(Slot::Register8(B), Slot::Register8(B))
        );
        assert_eq!(
            decode(&mut [0x5fu8].iter().copied()).unwrap(),
            Opcode::Ld(Slot::Register8(E), Slot::Register8(A))
        );
        assert_eq!(
            decode(&mut [0x66u8].iter().copied()).unwrap(),
            Opcode::Ld(Slot::Register8(H), Slot::AddrRegister(AddrRegister::HL),)
        );
        assert_eq!(
            decode(&mut [0x68u8].iter().copied()).unwrap(),
            Opcode::Ld(Slot::Register8(L), Slot::Register8(B)),
        );

        assert_eq!(
            decode(&mut [0x7du8].iter().copied()).unwrap(),
            Opcode::Ld(Slot::Register8(A), Slot::Register8(L)),
        );
        assert_eq!(decode(&mut [0x76u8].iter().copied()).unwrap(), Opcode::Halt);
    }
}
