# GB disassembler

The long-term goal is to build an emulator for the Game Boy. But as a first step, I am going to build a disassembler to reverse engineer the Gameboy Boot ROM

## Gameboy Boot ROM

- The Boot rom I am disassembling has the md5 `32fbbd84168d3482956eb3c5051637f5` and should be the [DMG](https://gbdev.io/pandocs/Power_Up_Sequence.html) version.
- See `boot.ann` for the annotations

### Annotations

The annotation file has a simple format:

```
0xOFFSET PURPOSE VALUE
0xOFFSET PURPOSE VALUE
```

- OFFSET is the hex offset of the OPCODE from the beginning of the file
- PURPOSE can be C (comment), G (goto, jump), L (label for a jump), S (section). Section text will appear before the line.
- VALUE is what will be displayed for the current OPCODE

### Disassemble

Run this command to display the disassembled code with the annotations:

```shell
cargo run boot.gb boot.ann
```

# Resources

Opcodes: https://meganesu.github.io/generate-gb-opcodes/

# The bible

- Root: https://gbdev.io/pandocs
- Registers: https://gbdev.io/pandocs/CPU_Registers_and_Flags.html
- Memory layout: https://gbdev.io/pandocs/Memory_Map.html

## Registers
Since I am reverse-engineering the boot rom, it is handy to have a list of all special memory mapped addresses. This list has been made based on  http://bgb.bircd.org/pandocs.htm


```
0000-3FFF   16KB ROM Bank 00     (in cartridge, fixed at bank 00)
 - BOOT rom (during boot): 0000-00ff
 - Jump vectors (RST): 0000,0008,0010,0018,0020,0028,0030,0038
 - Jump vectors (Interrupts): 0040,0048,0050,0058,0060
 - Cartridge header: 0100-014F
4000-7FFF   16KB ROM Bank 01..NN (in cartridge, switchable bank number)
8000-9FFF   8KB Video RAM (VRAM) (switchable bank 0-1 in CGB Mode)
 - 8000-8FFF - Tile pattern #1
 - 8800-97FF - Tile pattern #2
 - 9800-9BFF - background map #1 (see FF40 - LCDC)
 - 9C00-9FFF - background map #2 (see FF40 - LCDC)
A000-BFFF   8KB External RAM     (in cartridge, switchable bank, if any)
C000-CFFF   4KB Work RAM Bank 0 (WRAM)
D000-DFFF   4KB Work RAM Bank 1 (WRAM)  (switchable bank 1-7 in CGB Mode)
E000-FDFF   Same as C000-DDFF (ECHO)    (typically not used)
FE00-FE9F   Sprite Attribute Table (OAM)
FEA0-FEFF   Not Usable
FF00-FF7F   I/O Ports
 - FF00 - P1/JOYP - Joypad (R/W)
 - FF01 - SB - Serial transfer data (R/W)
 - FF02 - SC - Serial Transfer Control (R/W)
 - FF04 - DIV - Divider Register (R/W)
 - FF05 - TIMA - Timer counter (R/W)
 - FF06 - TMA - Timer Modulo (R/W)
 - FF07 - TAC - Timer Control (R/W)
 - FF10 - NR10 - Channel 1 Sweep register (R/W)
 - FF11 - NR11 - Channel 1 Sound length/Wave pattern duty (R/W)
 - FF12 - NR12 - Channel 1 Volume Envelope (R/W)
 - FF13 - NR13 - Channel 1 Frequency lo (Write Only)
 - FF14 - NR14 - Channel 1 Frequency hi (R/W)
 - FF16 - NR21 - Channel 2 Sound Length/Wave Pattern Duty (R/W)
 - FF17 - NR22 - Channel 2 Volume Envelope (R/W)
 - FF18 - NR23 - Channel 2 Frequency lo data (W)
 - FF19 - NR24 - Channel 2 Frequency hi data (R/W)
 - FF1A - NR30 - Channel 3 Sound on/off (R/W)
 - FF1B - NR31 - Channel 3 Sound Length
 - FF1B - NR31 - Channel 3 Sound Length
 - FF1D - NR33 - Channel 3 Frequency's lower data (W)
 - FF1E - NR34 - Channel 3 Frequency's higher data (R/W)
 - FF20 - NR41 - Channel 4 Sound Length (R/W)
 - FF21 - NR42 - Channel 4 Volume Envelope (R/W)
 - FF22 - NR43 - Channel 4 Polynomial Counter (R/W)
 - FF23 - NR44 - Channel 4 Counter/consecutive; Inital (R/W)
 - FF24 - NR50 - Channel control / ON-OFF / Volume (R/W)
 - FF25 - NR51 - Selection of Sound output terminal (R/W)
 - FF26 - NR52 - Sound on/off
 - FF30-FF3F - Wave Pattern RAM
 - FF40 - LCDC - LCD Control (R/W)
 - FF41 - STAT - LCDC Status (R/W)
 - FF42 - SCY - Scroll Y (R/W)
 - FF43 - SCX - Scroll X (R/W)
 - FF44 - LY - LCDC Y-Coordinate (R)
 - FF45 - LYC - LY Compare (R/W)
 - FF46 - DMA - DMA Transfer and Start Address (W)
 - FF47 - BGP - BG Palette Data (R/W) - Non CGB Mode Only
 - FF48 - OBP0 - Object Palette 0 Data (R/W) - Non CGB Mode Only
 - FF49 - OBP1 - Object Palette 1 Data (R/W) - Non CGB Mode Only
 - FF4A - WY - Window Y Position (R/W)
 - FF4B - WX - Window X Position minus 7 (R/W)
FF80-FFFE   High RAM (HRAM)
FF0F - IF - Interrupt Flag (R/W)
FFFF        Interrupt Enable Register
```