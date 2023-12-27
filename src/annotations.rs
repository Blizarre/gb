use std::{
    collections::BTreeMap, error::Error, fmt::Display, fs::File, io::Read, num::ParseIntError,
};

use itertools::Itertools;

#[derive(PartialEq, Debug, Clone)]
pub enum Purpose {
    Comment,
    Section,
    Goto,
    Label,
    Data,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Annotation {
    pub location: usize,
    pub purpose: Purpose,
    pub value: String,
}

impl Purpose {
    fn from_char(mnemonic: &str) -> Result<Self, AnnotationError> {
        Ok(match mnemonic {
            "C" => Purpose::Comment,
            "S" => Purpose::Section,
            "G" => Purpose::Goto,
            "L" => Purpose::Label,
            "D" => Purpose::Data,
            _ => return Err(AnnotationError::InvalidMnemonic(mnemonic.to_string())),
        })
    }
}

impl Annotation {
    pub fn parse(data: &str) -> Result<BTreeMap<usize, Vec<Annotation>>, AnnotationError> {
        let annotations = data
            .split('\n')
            .filter(|l| !l.trim().is_empty())
            .filter(|l| !l.starts_with('#'))
            .map(Annotation::from_line)
            .collect::<Result<Vec<Annotation>, AnnotationError>>()?;

        Ok(annotations
            .iter()
            .sorted_by_key(|a| a.location)
            .group_by(|a| a.location)
            .into_iter()
            .map(|(key, group)| (key, group.cloned().collect()))
            .collect::<BTreeMap<usize, Vec<Annotation>>>())
    }

    pub fn parse_file(
        file_name: &String,
    ) -> Result<BTreeMap<usize, Vec<Annotation>>, AnnotationError> {
        let mut tmp = String::new();
        File::open(file_name).and_then(|mut f| f.read_to_string(&mut tmp))?;
        Self::parse(&tmp)
    }

    fn from_line(line: &str) -> Result<Self, AnnotationError> {
        let items: Vec<&str> = line.splitn(3, ' ').collect();
        if items.len() != 3 {
            Err(AnnotationError::MissingField)
        } else {
            Ok(Annotation {
                location: usize::from_str_radix(items[0].trim_start_matches("0x"), 16)?,
                purpose: Purpose::from_char(items[1])?,
                value: items[2].to_string(),
            })
        }
    }
}

#[derive(Debug)]
pub enum AnnotationError {
    MissingField,
    InvalidMnemonic(String),
    IOError(std::io::Error),
    ParseError(ParseIntError),
}

impl Error for AnnotationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::MissingField => None,
            Self::InvalidMnemonic(_m) => None,
            Self::IOError(err) => Some(err),
            Self::ParseError(err) => Some(err),
        }
    }
}

impl From<ParseIntError> for AnnotationError {
    fn from(value: ParseIntError) -> Self {
        AnnotationError::ParseError(value)
    }
}

impl From<std::io::Error> for AnnotationError {
    fn from(value: std::io::Error) -> Self {
        AnnotationError::IOError(value)
    }
}

impl Display for AnnotationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingField => f.write_str("Missing Field in Annotation"),
            Self::InvalidMnemonic(m) => write!(f, "Invalid Mnemonic {}", m),
            Self::IOError(err) => write!(f, "IO Error {}", err),
            Self::ParseError(err) => write!(f, "Parse error: {}", err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn test_purpose_from() {
        assert_eq!(Purpose::from_char("C").unwrap(), Purpose::Comment);
        assert_eq!(Purpose::from_char("S").unwrap(), Purpose::Section);
        assert_eq!(Purpose::from_char("G").unwrap(), Purpose::Goto);
        assert_eq!(Purpose::from_char("L").unwrap(), Purpose::Label);
        assert_eq!(Purpose::from_char("D").unwrap(), Purpose::Data);
    }

    #[test]
    fn test_purpose_from_invalid() {
        assert!(matches!(
            Purpose::from_char("Q").unwrap_err(),
            AnnotationError::InvalidMnemonic(_err)
        ));
    }

    #[test]
    fn test_annotation_from() {
        let line = "0x1234 C some comment";
        let expected = Annotation {
            location: 0x1234,
            purpose: Purpose::Comment,
            value: "some comment".to_string(),
        };
        assert_eq!(Annotation::from_line(line).unwrap(), expected);
    }

    #[test]
    fn test_annotation_from_invalid_line() {
        let line = "0x1234 C";
        assert!(Annotation::from_line(line).is_err());
    }

    #[test]
    fn test_annotation_parse() {
        let data = "0x1234 C comment\n0x5678 S section".to_string();
        let mut expected = BTreeMap::new();
        expected.insert(
            0x1234,
            vec![Annotation {
                location: 0x1234,
                purpose: Purpose::Comment,
                value: "comment".to_string(),
            }],
        );
        expected.insert(
            0x5678,
            vec![Annotation {
                location: 0x5678,
                purpose: Purpose::Section,
                value: "section".to_string(),
            }],
        );
        assert_eq!(Annotation::parse(&data).unwrap(), expected);
    }

    #[test]
    fn test_annotation_parse_invalid_data() {
        let data = "0x1234 C value\n0x567w S test".to_string();
        assert!(matches!(
            Annotation::parse(&data).unwrap_err(),
            AnnotationError::ParseError(_err)
        ));

        let data = "0x1234 C\n0x567a S test".to_string();
        assert!(matches!(
            Annotation::parse(&data).unwrap_err(),
            AnnotationError::MissingField
        ));

        let data = "0x1234 C test\n0x567a W test".to_string();
        assert!(matches!(
            Annotation::parse(&data).unwrap_err(),
            AnnotationError::InvalidMnemonic(_err)
        ));
    }
}
