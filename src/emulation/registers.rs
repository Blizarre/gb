use std::fmt::Display;

#[derive(Default)]
pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    //
    // Special registers
    pub f: u8,   // flags
    pub pc: u16, // program counter
    pub sp: u16, // stack pointer
}

impl Registers {
    pub fn flag_zero(&self) -> bool {
        (self.f & (1 << 7)) != 0
    }


    pub fn flag_zero_u8(&self) -> u8 {
        self.f & (1 << 7)
    }

    pub fn flag_zero_set(&mut self, value: bool) {
        self.f = if value {
            self.f | (1 << 7)
        } else {
            self.f & (!(1 << 7))
        }
    }

    pub fn flag_carry(&self) -> bool {
        (self.f & (1 << 4)) != 0
    }

    /// Carry flag in the lsb
    pub fn flag_carry_u8(&self) -> u8 {
        (self.f & (1 << 4)) >> 4
    }

    pub fn flag_carry_set(&mut self, value: bool) {
        self.f = if value {
            self.f | (1 << 4)
        } else {
            self.f & (!(1 << 4))
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
        write!(f, "pc: {:4x} f: {:2x} sp: {:4x}", self.pc, self.f, self.sp)
    }
}

impl Registers {
    pub fn af(&self) -> u16 {
        (self.a as u16) << 8 | self.f as u16
    }

    pub fn bc(&self) -> u16 {
        (self.b as u16) << 8 | self.c as u16
    }

    pub fn de(&self) -> u16 {
        (self.d as u16) << 8 | self.e as u16
    }

    pub fn hl(&self) -> u16 {
        (self.h as u16) << 8 | self.l as u16
    }

    pub fn set_hl(&mut self, val: u16) {
        self.h = (val >> 8) as u8;
        self.l = (val & 0xFF) as u8;
    }
}

#[cfg(test)]
mod tests {
    use crate::emulation::registers::Registers;

    #[test]
    fn test_flag_zero() {
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
    fn test_flag_carry() {
        let mut registers = Registers::default();
        registers.f = 0b10111010;
        assert!(registers.flag_carry());
        registers.flag_carry_set(false);
        assert!(!registers.flag_carry());
        assert_eq!(registers.f, 0b10101010);
        registers.flag_carry_set(true);
        assert!(registers.flag_carry());
        assert_eq!(registers.f, 0b10111010);
    }
}
