#!/usr/bin/fish
cargo build --release
objcopy -S -O srec target/thumbv7em-none-eabi/release/md407-test md407-test.s19
md407 load md407-test.s19
