set shell := ["sh", "-c"]

build:
    cargo build
    rm target/powerpc-unknown-eabi/debug/rogue.elf
    cp target/powerpc-unknown-eabi/debug/rogue target/powerpc-unknown-eabi/debug/rogue.elf

run: build
    dolphin-emu -e target/powerpc-unknown-eabi/debug/rogue.elf
