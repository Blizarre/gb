use clap::{Arg, ArgAction, Command};
use std::collections::BTreeMap;
use std::{error::Error, fs::File, io::Read};

extern crate gb;

use gb::annotations::{Annotation, Purpose};
use gb::decoder::{decode, Memory, Opcode};

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
    let mut index = 0u16;
    let memory = Memory::from_raw(&data)?;

    loop {
        let mut comment = String::new();
        let mut goto = String::new();
        let mut label = None;
        let mut skip = 0u16;
        let annotations = annotations.get(&(index as usize)).unwrap_or(&empty_vec);

        for annotation in annotations {
            match annotation.purpose {
                Purpose::Comment => comment = format!(" ; {}", &annotation.value),
                Purpose::Goto => goto = format!("-> {}", &annotation.value),
                Purpose::Label => label = Some(annotation.value.to_string()),
                Purpose::Section => {
                    println!("\n-- {} --", annotation.value)
                }
                Purpose::Data => {
                    skip =
                        u16::from_str_radix(annotation.value.trim_start_matches("0x"), 16).unwrap();
                }
            }
        }

        if let Some(l) = label {
            println!("{}:", l);
        }
        if skip > 0 {
            println!(
                "Skip 0x{:04x}-0x{:04x} {} {}",
                index,
                index + skip - 1,
                goto,
                comment
            );
            index += skip - 1;
        } else {
            let current_index = index;

            let opcode = decode(&memory, &mut index).unwrap();
            if debug {
                print!("{:02x} ", memory.get(current_index));
            }
            // Display the destination address of a jump if it has not been provided
            goto = if goto.is_empty() {
                let fmt_offset =
                    |offset| format!("-> 0x{:x}", current_index as isize + offset as isize);
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
