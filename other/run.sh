#!/bin/sh
cp ../target/powerpc-unknown-eabi/debug/rogue rogue.elf
dolphin-emu -e rogue.elf
