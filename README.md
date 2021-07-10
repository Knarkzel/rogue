# Rogue

Example game made in Rust using `ogc-rs`, targeting Wii.

## Setup

- `Rust` toolchain is required. Follow https://rustup.rs/.
- You must have `devkitPro` installed. See [https://devkitpro.org/wiki/Getting_Started].
- `CLANG_VERSION` must be set to your clang version, derived from `clang -v`. For instance: `CLANG_VERSION="12.0.0"`.
- `just` is used for running in `dolphin-emu`. `cargo install just`.

## Running

```sh
git clone https://github.com/knarkzel/rogue
cd rogue
just run
```
