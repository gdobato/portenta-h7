# portenta-h7

[![ci](https://github.com/gdobato/portenta-h7/actions//workflows/ci.yml/badge.svg)](https://github.com/gdobato/portenta-h7/actions/workflows/ci.yml) 

portenta-h7 provides examples for the Arduino Portenta-H7 board written in Rust. The entry point address for the application is located at **0x08040000** to which the Arduino bootloader jumps. The software can be flashed on the target either with USB (DFU), or with a debug probe (JLink, ST-Link). Flashing with Arduino IDE is not supported.
## Installation (Unix-like OS)
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add thumbv7em-none-eabihf
cargo install cargo-embed cargo-binutils
```

## Build
To build an example, run the following command:
```
cargo <example_name> [--release]
```
For instance, to build `blinky`:
```
cargo blinky
```
## Flash with DFU (USB)
1. Install [dfu-utils](https://dfu-util.sourceforge.net/) on your system.
2. Connect USB to Portenta.
3. Set the Portenta in bootloader mode by pressing the reset button twice.
4. Generate the target binary by running the following command:
   ```
   cargo <example_name>-bin
   ```
   For example, to generate the target binary for `blinky`, run the following command:
   ```
   cargo blinky-bin
   ```
4. Flash the binary to the target by running the following command:
   ```
   dfu-util -a 0 -d 2341:035b --dfuse-address=0x08040000:leave -D <binary_path>
   ```
   For example, to flash `blinky`, run the following command:
   ```
   dfu-util -a 0 -d 2341:035b --dfuse-address=0x08040000:leave -D target/thumbv7em-none-eabihf/release/examples/blinky.bin
   ```
## Flash with debug probe (JLink, ST-Link)
1. Connect the probe to the JTAG port of the breakout board.
2. Run the following command:
   ```
   cargo  <example_name>-probe [--release]
   ```
   For example, to flash `blinky`, run the following command:
   ```
   cargo blinky-probe
   ```