[![Cargo Build](https://github.com/IonImpulse/HMMM-Rust/actions/workflows/rust.yml/badge.svg)](https://github.com/IonImpulse/HMMM-Rust/actions/workflows/rust.yml)
# hmmm_rs
```
██    ██  ████    ████  ████    ████  ████    ████
██    ██  ██ ██  ██ ██  ██ ██  ██ ██  ██ ██  ██ ██
████████  ██  ████  ██  ██  ████  ██  ██  ████  ██
██    ██  ██   ██   ██  ██   ██   ██  ██   ██   ██
██    ██  ██        ██  ██        ██  ██        ██
 HARVEY       MUDD       MINIATURE      MACHINE
```
A Rust-based compiler, decompiler, debugger, and simulator for the [Harvey Mudd Miniature Machine (HMMM)](https://www.cs.hmc.edu/~cs5grad/cs5/hmmm/documentation/documentation.html)

This program aims to be used as a drop-in upgrade to the original Python script written to use this "assembly" language. 
It assembles and executes HMMM in the same way, and can read .hmmm and .hb files produced by the original script, as well as decompile a .hb file to a human-readable .hmmm file.
The UI is completely different, with **colors** and *italics*, and the debugging mode has been vastly improved to show more infomation in a much more human-readable manner.

Currently, this project is still in **alpha**, so bugs may pop up here and there, but most of the functionality should work fine.
# How to use it:
```
USAGE:
    HMMM-Rust.exe [FLAGS] [OPTIONS]

FLAGS:
    -d, --debug      Use debug mode for stepping through simulator
    -h, --help       Prints help information
    -n, --no-run     Do not simulate (run) the program on compilation
    -V, --version    Prints version information

OPTIONS:
    -i, --input <input>      Input .hmmm or .hb file
    -o, --output <output>    Output location of either .hmmm or .hb file
```

Just run a .hmmm file: `.\HMMM-Rust -i tests\test.hmmm`

Run a .hmmm file and save the compiled binary: `.\HMMM-Rust -i tests\test.hmmm -o compiled.hb`

Decompile a .hb file and save it as a .hmmm file: `.\HMMM-Rust -i compiled.hb -o tests\test.hmmm`

NOTE: compiling to a .hmmm file to a .hb file and then decompiling to a .hmmm file will result in the same program, but comments in the original .hmmm file will be lost.
