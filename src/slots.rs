use std::fmt::Debug;

#[derive(Debug)]
pub enum AddrRegister {
    BC,
    DE,
    HL,
    C,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Register16 {
    BC,
    DE,
    FG,
    HL,
    SP,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Register8 {
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

#[allow(dead_code)]
pub enum Slot {
    AddrRegister(AddrRegister),
    Register16(Register16),
    Register8(Register8),
    Addr8(u8),
    Addr16(u16),
    Data8(u8),
    Data16(u16),
}

impl Debug for Slot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Register8(val) => write!(f, "{:?}", val),
            Self::Register16(val) => write!(f, "{:?}", val),
            Self::AddrRegister(val) => write!(f, "({:?})", val),
            Self::Addr8(val) => write!(f, "(0x{:02x})", val),
            Self::Addr16(val) => write!(f, "(0x{:04x})", val),
            Self::Data8(val) => write!(f, "0x{:02x}", val),
            Self::Data16(val) => write!(f, "0x{:04x}", val),
        }
    }
}

impl Slot {
    #[allow(dead_code)]
    pub fn parse_a16(data: &[u8]) -> Self {
        Slot::Addr16(decode_u16(data))
    }

    pub fn parse_a8(data: &[u8]) -> Self {
        Slot::Addr8(data[0])
    }

    pub fn parse_d16(data: &[u8]) -> Self {
        Slot::Data16(decode_u16(data))
    }

    pub fn parse_d8(data: &[u8]) -> Self {
        Slot::Data8(data[0])
    }

    pub fn r8(r: Register8) -> Slot {
        Slot::Register8(r)
    }

    pub fn r16(r: Register16) -> Slot {
        Slot::Register16(r)
    }
}

fn decode_u16(data: &[u8]) -> u16 {
    u16::from_le_bytes(
        data[..2]
            .try_into()
            .expect("Enf of file in the middle of a constant"),
    )
}
