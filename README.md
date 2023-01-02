# portenta-rs [WIP]

[![ci](https://github.com/gdobato/portenta-rs/actions//workflows/ci.yml/badge.svg)](https://github.com/gdobato/portenta-rs/actions/workflows/ci.yml) 

Sketches for Arduino Portenta-H7 written in Rust, in which some of the available embedded-rust crates are used.

DAP access via **SWD** is required to flash it on the target. It cannot be flashed via Arduino IDE or any other related framework which uses DFU.

Entry point address is located at **0x08040000**, at which arduino bootloader jumps to.

**⚠️  Make sure you do not override the Arduino bootloader since it is needed for some early stage initializations, like the PMIC configuration.
Overriding Arduino bootloader could lead to a bricked board. Use this project at your own risk.**

### Installation (Unix-like OS)
Toolchain
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

rustup target add thumbv7em-none-eabihf
```
[Cargo-embed](https://github.com/probe-rs/cargo-embed)
```
cargo install cargo-embed
```

### Build

```
cargo build [--release] --example <example_name>
```
e.g :
```
cargo build --example led_blinky
```
### Flash on target
```
cargo embed [--release] --example <example_name>
```
e.g :
```
cargo embed --example led_blinky
```