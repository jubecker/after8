# After8

Emulator for Chip-8 roms implemented in Rust. The UI is rendered as console output.

## Chip-8 references

- [Wikipedia](https://en.wikipedia.org/wiki/CHIP-8)
- Collections of Chip-8 references: [Awesome Chip-8](https://github.com/tobiasvl/awesome-chip-8)
- [Op codes](https://chip8.gulrak.net/)
- [Test suite](https://github.com/Timendus/chip8-test-suite)

## Building

A working rust setup is needed to build the emulator. See [rustup](https://rustup.rs/) to get started.

```
cargo build --release
```

This creates the after8 binary in 'target/release/'.

## Fetching test roms

Run the supplied script in the 'roms/' directory to download test roms.

```
./roms/download_roms.sh
```

## Running

To run a rom file, pass it as the last argument.

```
./target/release/after8 roms/3-corax+.ch8
```

To get debug output (-v) and disable the UI (-u), run

```
./target/release/after8 -v -u roms/3-corax+.ch8
```

## Not yet implemented

- keyboard input
- graphical UI
