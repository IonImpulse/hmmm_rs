[![Cargo Build](https://github.com/IonImpulse/HMMM-Rust/actions/workflows/rust.yml/badge.svg)](https://github.com/IonImpulse/HMMM-Rust/actions/workflows/rust.yml)
# HMMM RS
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
    hmmm_rs.exe [FLAGS] [OPTIONS]

FLAGS:
    -d, --debug      Use debug mode for stepping through simulator
    -h, --help       Prints help information
    -n, --no-run     Do not simulate (run) the program on compilation
    -V, --version    Prints version information

OPTIONS:
    -a, --autograder <autograder>    Toggles the AutoGrader functionality, expecting a test string to be given. If
                                     enabled, expects a directory path instead of a file path for --input and --output.
                                     --debug, --no-run, and --speed are ignored in this mode.
    -i, --input <input>              Input .hmmm or .hb file
    -o, --output <output>            Output location of either .hmmm or .hb file
    -s, --speed <speed>              Sets the multiplier (speed) of debug mode (eg: .5 is half speed, 2 is double)
```

Just run a .hmmm file: `.\hmmm_rs -i tests\test.hmmm`

Run a .hmmm file and save the compiled binary: `.\hmmm_rs -i tests\test.hmmm -o compiled.hb`

Decompile a .hb file and save it as a .hmmm file: `.\hmmm_rs -i compiled.hb -o tests\test.hmmm`

NOTE: compiling to a .hmmm file to a .hb file and then decompiling to a .hmmm file will result in the same program, but comments in the original .hmmm file will be lost.

# AutoGrader Mode
![Grading output](https://user-images.githubusercontent.com/24578597/128656748-967b2df6-1725-4c72-942d-bda485c5fed2.png)


HMMM_RS comes with a built-in autograding system. By using the "--autograder" or "-a" flag, followed by some test cases, you can test an entire directory of
.hmmm files against your specifications. The test case string should be formatted as follows:
```
-a "input #1, input #2, ..., input #n | output #1, output #2, output #3, ..., output #n;"
EXAMPLE:
-a "16, 2 | 16, 8;"
```

The testcase "16, 2 | 16, 8;" means that the program should ask for input *exactly* twice, and will be given the values 16 and 2. Additionaly, it needs to output the numbers 16 and 8.

You can chain as many testcases as you want together, so long as you seperate them with a semicolon.
```
EXAMPLE:
-a "16, 2 | 16, 8; 15, 2 | 15, 7; 64, 3 | 64, 21;"
```
All three testcases will be run and graded against.

As HMMM can only output integers, only integers will be parsed correctly. Any other character will throw an error.

# System Exit Codes:
On exit, HMMM_RS produces a system exit code that matches the exit problem. This value can be read by a process calling it, providing a method for external tools to compile/run HMMM. For a program successfully exiting, a error code of `0` is produced. The rest are as follows:
## Compile Errors:
```
InstructionDoesNotExist:  ->  10
InvalidArgumentType:      ->  11
InvalidRegister:          ->  12
TooManyArguments:         ->  13
TooFewArguments:          ->  14
InvalidSignedNumber:      ->  15
InvalidUnsignedNumber:    ->  16
InvalidNumber:            ->  17
CorruptedBinary:          ->  18
LineNumberNotPresent:     ->  19
InvalidLineNumber:        ->  20
```
## Runtime Errors:
```
InvalidRegisterLocation:  ->  100
MemoryLocationNotData:    ->  101
InvalidMemoryData:        ->  102
InvalidMemoryLocation:    ->  103
InvalidData:              ->  104
InvalidSignedNumber:      ->  105
InvalidProgramCounter:    ->  106
InstructionIsData:        ->  107
InvalidInstructionType:   ->  108
DivideByZero:             ->  109
RegisterOutOfBounds:      ->  110
```

# Table of Instructions
Instruction taken from official [HMMM documentation](https://www.cs.hmc.edu/~cs5grad/cs5/hmmm/documentation/documentation.html).

|        ***Instruction***            | ***Description***                                                          |    ***Aliases***    |
|:-----------------------------:|----------------------------------------------------------------------|:-------------:|
| **System instructions**           |                                                                      |               |
| halt                          | Stop!                                                                | None          |
| read rX                       | Place user input in register rX                                      | None          |
| write rX                      | Print contents of register rX                                        | None          |
| nop                           | Do nothing                                                           | None          |
| **Setting register data**         |                                                                      |               |
| setn rX N                     | Set register rX equal to the integer N (-128 to +127)                | None          |
| addn rX N                     | Add integer N (-128 to 127) to register rX                           | None          |
| copy rX rY                    | Set rX = rY                                                          | mov           |
| **Arithmetic**                    |                                                                      |               |
| add rX rY rZ                  | Set rX = rY + rZ                                                     | None          |
| sub rX rY rZ                  | Set rX = rY - rZ                                                     | None          |
| neg rX rY                     | Set rX = -rY                                                         | None          |
| mul rX rY rZ                  | Set rX = rY * rZ                                                     | None          |
| div rX rY rZ                  | Set rX = rY // rZ (integer division; rounds down; no remainder)      | None          |
| mod rX rY rZ                  | Set rX = rY % rZ (returns the remainder of integer division)         | None          |
| jumpn N                       | Set program counter to address N                                     | None          |
| jumpr rX                      | Set program counter to address in rX                                 | jump          |
| jeqzn rX N                    | If rX == 0, then jump to line N                                      | jeqz          |
| jnezn rX N                    | If rX != 0, then jump to line N                                      | jnez          |
| jgtzn rX N                    | If rX > 0, then jump to line N                                       | jgtz          |
| jltzn rX N                    | If rX < 0, then jump to line N                                       | jltz          |
| calln rX N                    | Copy addr. of next instr. into rX and then jump to mem. addr. N      | call          |
| **Interacting with memory (RAM)** |                                                                      |               |
| pushr rX rY                   | Store contents of register rX onto stack pointed to by reg. rY       | None          |
| popr rX rY                    | Load contents of register rX from stack pointed to by reg. rY        | None          |
| loadn rX N                    | Load register rX with the contents of memory address N               | None          |
| storen rX N                   | Store contents of register rX into memory address N                  | None          |
| loadr rX rY                   | Load register rX with data from the address location held in reg. rY | loadi, load   |
| storer rX rY                  | Store contents of register rX into memory address held in reg. rY    | storei, store |
