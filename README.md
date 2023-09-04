# GB disassembler

The long-term goal is to build an emulator for the Game Boy. But as a first step, I am going to build a disassembler to reverse engineer the Gameboy BIOS

## Gameboy BIOS

- The BIOS I am disassembling has the md5 `32fbbd84168d3482956eb3c5051637f5` and should be the [DMG](https://gbdev.io/pandocs/Power_Up_Sequence.html) version.
- See `bios.ann` for the annotations

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
cargo run bios.gb bios.ann
```

# Resources

Opcodes: https://meganesu.github.io/generate-gb-opcodes/

# The bible

- Root: https://gbdev.io/pandocs
- Registers: https://gbdev.io/pandocs/CPU_Registers_and_Flags.html
- Memory layout: https://gbdev.io/pandocs/Memory_Map.html
