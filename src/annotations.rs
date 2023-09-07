use std::{collections::BTreeMap, fs::File, io::Read};

use itertools::Itertools;

#[derive(PartialEq)]
pub enum Purpose {
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

impl Annotation {
    pub fn parse(file_name: &String) -> Result<BTreeMap<usize, Vec<Annotation>>, std::io::Error> {
        let mut tmp = String::new();
        File::open(file_name)
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
}

pub struct Annotation {
    pub location: usize,
    pub purpose: Purpose,
    pub value: String,
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
