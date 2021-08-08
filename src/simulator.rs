use super::*;
use colored::*;
use lazy_static::lazy_static;
use std::io;
use std::io::stdin;
use std::io::BufRead;
use terminal::*;

lazy_static! {
    static ref INSTRUCTION_LOOKUP: Vec<InstructionType> = vec![
        InstructionType::new(
            vec!["halt"],
            "0000 0000 0000 0000",
            "1111 1111 1111 1111",
            "",
            "Halts the program"
        ),
        InstructionType::new(
            vec!["read"],
            "0000 0000 0000 0001",
            "1111 0000 1111 1111",
            "r",
            "Place 16-bit integer in register _",
        ),
        InstructionType::new(
            vec!["write"],
            "0000 0000 0000 0010",
            "1111 0000 1111 1111",
            "r",
            "Print contents of register _"
        ),
        InstructionType::new(
            vec!["jumpr", "jump"],
            "0000 0000 0000 0011",
            "1111 0000 1111 1111",
            "r",
            "Set program counter to address in register _"
        ),
        InstructionType::new(
            vec!["setn"],
            "0001 0000 0000 0000",
            "1111 0000 0000 0000",
            "rs",
            "Set register _ equal to integer _"
        ),
        InstructionType::new(
            vec!["loadn"],
            "0010 0000 0000 0000",
            "1111 0000 0000 0000",
            "ru",
            "Load register _ with contents of memory address _"
        ),
        InstructionType::new(
            vec!["storen"],
            "0011 0000 0000 0000",
            "1111 0000 0000 0000",
            "ru",
            "Place contents of register _ into memory address _"
        ),
        InstructionType::new(
            vec!["loadr", "loadi", "load"],
            "0100 0000 0000 0000",
            "1111 0000 0000 0000",
            "rr",
            "Load register _ with memory data indexed by register _"
        ),
        InstructionType::new(
            vec!["storer", "storei", "store"],
            "0100 0000 0000 0001",
            "1111 0000 0000 0000",
            "rr",
            "Store register _ in memory indexed by register _"
        ),
        InstructionType::new(
            vec!["popr"],
            "0100 0000 0000 0010",
            "1111 0000 0000 1111",
            "rr",
            "Subtract 1 from the indexing register, then loadr"
        ),
        InstructionType::new(
            vec!["pushr"],
            "0100 0000 0000 0011",
            "1111 0000 0000 1111",
            "rr",
            "storer, then add 1 to the indexing register"
        ),
        InstructionType::new(
            vec!["addn"],
            "0101 0000 0000 0000",
            "1111 0000 0000 0000",
            "rs",
            "Take register _ and add _ to it"
        ),
        InstructionType::new(
            vec!["nop"],
            "0110 0000 0000 0000",
            "1111 1111 1111 1111",
            "",
            "Do nothing"
        ),
        InstructionType::new(
            vec!["copy", "mov"],
            "0110 0000 0000 0000",
            "1111 0000 0000 1111",
            "rr",
            "Set register _ = register _"
        ),
        InstructionType::new(
            vec!["add"],
            "0110 0000 0000 0000",
            "1111 0000 0000 0000",
            "rrr",
            "Set register _ = register _ + register _"
        ),
        InstructionType::new(
            vec!["neg"],
            "0111 0000 0000 0000",
            "1111 0000 1111 0000",
            "rzr",
            "Set register _ = - register _"
        ),
        InstructionType::new(
            vec!["sub"],
            "0111 0000 0000 0000",
            "1111 0000 0000 0000",
            "rrr",
            "Set register _ = register _ - register _"
        ),
        InstructionType::new(
            vec!["mul"],
            "1000 0000 0000 0000",
            "1111 0000 0000 0000",
            "rrr",
            "Set register _ = register _ * register _"
        ),
        InstructionType::new(
            vec!["div"],
            "1001 0000 0000 0000",
            "1111 0000 0000 0000",
            "rrr",
            "Set register _ = register _ // register _ (int. division)"
        ),
        InstructionType::new(
            vec!["mod"],
            "1010 0000 0000 0000",
            "1111 0000 0000 0000",
            "rrr",
            "Set register _ = register _ % register _ (remainder of div.)"
        ),
        InstructionType::new(
            vec!["jumpn"],
            "1011 0000 0000 0000",
            "1111 1111 0000 0000",
            "zu",
            "Set program counter to address _"
        ),
        InstructionType::new(
            vec!["calln", "call"],
            "1011 0000 0000 0000",
            "1111 0000 0000 0000",
            "ru",
            "Copy address of next instruction into register _, and jump to address _"
        ),
        InstructionType::new(
            vec!["jeqzn", "jeqz"],
            "1100 0000 0000 0000",
            "1111 0000 0000 0000",
            "ru",
            "If register _ == 0, jump to line _"
        ),
        InstructionType::new(
            vec!["jnezn", "jnez"],
            "1101 0000 0000 0000",
            "1111 0000 0000 0000",
            "ru",
            "If register _ != 0, jump to line _"
        ),
        InstructionType::new(
            vec!["jgtzn", "jgtz"],
            "1110 0000 0000 0000",
            "1111 0000 0000 0000",
            "ru",
            "If register _ > 0, jump to line _"
        ),
        InstructionType::new(
            vec!["jltzn", "jltz"],
            "1111 0000 0000 0000",
            "1111 0000 0000 0000",
            "ru",
            "If register _ < 0, jump to line _"
        ),
        InstructionType::new(
            vec!["data"],
            "0000 0000 0000 0000",
            "0000 0000 0000 0000",
            "n",
            "ERROR: DATA _"
        ),
    ]
    .into_iter()
    .collect();
}

/// Struct for all instructions types, to make it easier to
/// consolidate ones with aliases and order all of the
/// matching and masking strings
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct InstructionType {
    /// List of all names, with the first name being
    /// used as the default
    pub names: Vec<&'static str>,
    /// String that will match an instruction from
    /// a .hmmm file
    pub match_string: &'static str,
    /// String that will match where additional information
    /// such as numbers or registers lay
    pub mask_string: &'static str,
    /// Argument lookup:
    ///
    /// "r" : Register
    ///
    /// "s" : Signed 8-bit decimal
    ///
    /// "u" : Unsigned 8-bit decimal
    ///
    /// "n" : Sign/Unsigned 16-bit hex/decimal
    ///
    /// "z" : Skip 4 bits of 0s
    pub arguments: &'static str,
    /// String that briefly describes the instruction
    /// in a human-readable format, with "_" signifying
    /// where an argument can be inserted
    pub human_explanation: &'static str,
}

impl InstructionType {
    pub fn new(
        names: Vec<&'static str>,
        match_string: &'static str,
        mask_string: &'static str,
        arguments: &'static str,
        human_explanation: &'static str,
    ) -> InstructionType {
        InstructionType {
            names: names,
            match_string: match_string,
            mask_string: mask_string,
            arguments: arguments,
            human_explanation: human_explanation,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileErr {
    InstructionDoesNotExist,
    InvalidArgumentType,
    InvalidRegister,
    TooManyArguments,
    TooFewArguments,
    InvalidSignedNumber,
    InvalidUnsignedNumber,
    InvalidNumber,
    CorruptedBinary,
    LineNumberNotPresent,
    InvalidLineNumber,
}

impl CompileErr {
    pub fn as_code(&self) -> i32 {
        match self {
            CompileErr::InstructionDoesNotExist => 10,
            CompileErr::InvalidArgumentType => 11,
            CompileErr::InvalidRegister => 12,
            CompileErr::TooManyArguments => 13,
            CompileErr::TooFewArguments => 14,
            CompileErr::InvalidSignedNumber => 15,
            CompileErr::InvalidUnsignedNumber => 16,
            CompileErr::InvalidNumber => 17,
            CompileErr::CorruptedBinary => 18,
            CompileErr::LineNumberNotPresent => 19,
            CompileErr::InvalidLineNumber => 20,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Instruction {
    pub instruction_type: InstructionType,
    pub text_contents: String,
    pub binary_contents: Vec<String>,
}

impl Instruction {
    pub fn new_from_text(line_contents: &str) -> Result<Instruction, CompileErr> {
        // Split on both "," and " "
        let contents_list: Vec<&str> = line_contents.split(" ").collect();

        let mut instruction_type: Option<InstructionType> = None;

        for instruction in INSTRUCTION_LOOKUP.clone() {
            if instruction.names.contains(&contents_list[0]) {
                instruction_type = Some(instruction);
                break;
            }
        }

        // First, check to make sure the instruction type exists in the lookup table
        if instruction_type.is_none() {
            return Err(CompileErr::InstructionDoesNotExist);
        }

        let instruction_type = instruction_type.unwrap();

        let instruction_args: Vec<&str> = contents_list[1..].iter().map(|a| a as &str).collect();

        // Second, check to see if the number of arguments match
        if instruction_args.len() > instruction_type.arguments.len() {
            return Err(CompileErr::TooManyArguments);
        } else if instruction_args.len() < instruction_type.arguments.replace("z", "").len() {
            return Err(CompileErr::TooFewArguments);
        } else if instruction_type.arguments.len() == 0 {
            // If it's a single command, just return it
            return Ok(Instruction {
                instruction_type: instruction_type.clone(),
                text_contents: String::from(""),
                binary_contents: instruction_type
                    .clone()
                    .match_string
                    .split(" ")
                    .map(|a| String::from(a))
                    .collect(),
            });
        }

        let mut text_contents: String = String::from(instruction_args[0]);

        if instruction_args.len() == 2 {
            text_contents = format!("{} {}", text_contents, instruction_args[1]);
        } else {
            for i in 0..(instruction_args.len() - 1) {
                text_contents = format!("{}, {}", text_contents, instruction_args[1 + i]);
            }
        }

        let instruction_chars = instruction_type.arguments.chars();

        let mut binary_contents: Vec<String> = instruction_type
            .match_string
            .split(" ")
            .map(|a| String::from(a))
            .collect();

        let mut filled_slots: Vec<bool> = instruction_type
            .mask_string
            .split(" ")
            .map(|a| {
                if a == "0000" {
                    return false;
                } else {
                    return true;
                }
            })
            .collect();

        // Third, check if instructions match the source instruction types
        let mut arg_to_get = 0;
        for current_instruction_type in instruction_chars {
            let arg = instruction_args[arg_to_get];
            let slot_to_fill = filled_slots.iter().position(|a| *a == false).unwrap();
            let mut binary_string = String::from("");
            filled_slots[slot_to_fill] = true;

            if current_instruction_type == 'r' {
                if arg.to_lowercase().starts_with("r") {
                    let register_number = arg[1..].parse::<u8>();

                    if register_number.is_err() {
                        return Err(CompileErr::InvalidRegister);
                    }

                    binary_string = format!("{:04b}", register_number.unwrap());
                } else {
                    return Err(CompileErr::InvalidArgumentType);
                }
            } else if current_instruction_type == 's' {
                let number = arg.parse::<i8>();

                if number.is_err() {
                    return Err(CompileErr::InvalidSignedNumber);
                }

                binary_string = format!("{:08b}", number.unwrap());
            } else if current_instruction_type == 'u' {
                let number = arg.parse::<u8>();
                if number.is_err() {
                    return Err(CompileErr::InvalidUnsignedNumber);
                }

                binary_string = format!("{:08b}", number.unwrap());
            } else if current_instruction_type == 'n' {
                let number_dec = arg.parse::<i32>();
                let number_hex = i32::from_str_radix(arg, 16);

                if number_hex.is_ok() {
                    binary_string = format!("{:016b}", number_hex.unwrap());
                } else if number_dec.is_ok() {
                    binary_string = format!("{:016b}", number_dec.unwrap());
                } else {
                    return Err(CompileErr::InvalidNumber);
                }
            } else if current_instruction_type == 'z' {
                binary_string = "0000".to_string();
                arg_to_get -= 1;
                filled_slots[slot_to_fill] = false;
            }

            arg_to_get += 1;

            if binary_string.len() == 4 {
                binary_contents[slot_to_fill] = binary_string;
            } else if binary_string.len() == 8 {
                binary_contents[slot_to_fill] = String::from(binary_string.get(0..4).unwrap());
                binary_contents[slot_to_fill + 1] = String::from(binary_string.get(4..8).unwrap());
            } else {
                binary_contents[slot_to_fill] = String::from(binary_string.get(0..4).unwrap());
                binary_contents[slot_to_fill + 1] = String::from(binary_string.get(4..8).unwrap());
                binary_contents[slot_to_fill + 2] = String::from(binary_string.get(8..12).unwrap());
            }
        }

        Ok(Instruction {
            instruction_type: instruction_type,
            text_contents: text_contents,
            binary_contents: binary_contents,
        })
    }

    pub fn new_from_binary(line_contents: &str) -> Result<Instruction, CompileErr> {
        let binary_contents: Vec<String> = line_contents
            .clone()
            .split(" ")
            .map(|a| String::from(a))
            .collect();

        let mut instruction_type: Option<InstructionType> = None;

        let line_split: Vec<String> = line_contents.split(" ").map(|a| String::from(a)).collect();

        for instruction in INSTRUCTION_LOOKUP.clone().into_iter() {
            let mut matches_instruction: bool = true;

            let matcher: Vec<String> = instruction
                .match_string
                .split(" ")
                .map(|a| String::from(a))
                .collect();

            let mask: Vec<bool> = instruction
                .mask_string
                .split(" ")
                .map(|a| {
                    if a == "0000" {
                        return false;
                    } else {
                        return true;
                    }
                })
                .collect();

            for i in 0..4 {
                if mask[i] {
                    if matcher[i] != line_split[i] {
                        matches_instruction = false;
                    }
                }
            }

            if matches_instruction {
                instruction_type = Some(instruction);
                break;
            }
        }

        if instruction_type.is_none() {
            return Err(CompileErr::InstructionDoesNotExist);
        }

        let instruction_type = instruction_type.unwrap();
        let mut text_contents = String::from("");

        let mut instruction_args: Vec<String> = Vec::new();

        let mut slots_filled = 1;

        for arg_type in instruction_type.arguments.chars() {
            if arg_type == 'r' {
                instruction_args.push(format!(
                    "r{}",
                    u8::from_str_radix(binary_contents[slots_filled].as_str(), 2).unwrap()
                ));
                slots_filled += 1;
            } else if arg_type == 's' {
                let combined_binary = format!(
                    "{}{}",
                    binary_contents[slots_filled],
                    binary_contents[slots_filled + 1]
                );
                instruction_args.push(format!(
                    "{}",
                    i8::from_str_radix(combined_binary.as_str(), 2).unwrap()
                ));
                slots_filled += 2;
            } else if arg_type == 'u' {
                let combined_binary = format!(
                    "{}{}",
                    binary_contents[slots_filled],
                    binary_contents[slots_filled + 1]
                );
                instruction_args.push(format!(
                    "{}",
                    u8::from_str_radix(combined_binary.as_str(), 2).unwrap()
                ));
                slots_filled += 2;
            } else if arg_type == 'n' {
                let combined_binary = binary_contents.join("");

                instruction_args.push(format!(
                    "{}",
                    i32::from_str_radix(combined_binary.as_str(), 2).unwrap()
                ));
                slots_filled += 4;
            }
        }
        if instruction_args.len() > 0 {
            text_contents = String::from(instruction_args[0].clone());
        }
        if instruction_args.len() > 1 {
            for i in 1..(instruction_args.len()) {
                text_contents = format!("{}, {}", text_contents, instruction_args[i]);
            }
        }

        Ok(Instruction {
            instruction_type: instruction_type,
            text_contents: text_contents,
            binary_contents: binary_contents,
        })
    }

    pub fn new_data(data: &str) -> Self {
        Instruction {
            instruction_type: InstructionType::new(
                vec!["data"],
                "0000 0000 0000 0000",
                "0000 0000 0000 0000",
                "n",
                "Data",
            ),
            binary_contents: vec![
                data[0..4].to_string(),
                data[4..8].to_string(),
                data[8..12].to_string(),
                data[12..16].to_string(),
            ],
            text_contents: "".to_string(),
        }
    }

    pub fn new_blank_data() -> Self {
        Instruction::new_data("0000000000000000")
    }

    pub fn as_hex(self) -> String {
        let mut hex_string = "".to_string();

        for i in 0..4 {
            hex_string = format!(
                "{}{}",
                hex_string,
                format!(
                    "{:X}",
                    u8::from_str_radix(self.binary_contents[i].as_str(), 2).unwrap()
                )
            );
        }

        return hex_string;
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RuntimeErr {
    InvalidRegisterLocation,
    MemoryLocationNotData,
    InvalidMemoryData,
    InvalidMemoryLocation,
    InvalidData,
    InvalidSignedNumber,
    Halt,
    InvalidProgramCounter,
    InstructionIsData,
    InvalidInstructionType,
    DivideByZero,
    RegisterOutOfBounds,
    MaximumIterationsReached,
    TooManyInputs,
}

impl RuntimeErr {
    pub fn as_code(&self) -> i32 {
        match self {
            RuntimeErr::InvalidRegisterLocation => 100,
            RuntimeErr::MemoryLocationNotData => 101,
            RuntimeErr::InvalidMemoryData => 102,
            RuntimeErr::InvalidMemoryLocation => 103,
            RuntimeErr::InvalidData => 104,
            RuntimeErr::InvalidSignedNumber => 105,
            RuntimeErr::Halt => 0,
            RuntimeErr::InvalidProgramCounter => 106,
            RuntimeErr::InstructionIsData => 107,
            RuntimeErr::InvalidInstructionType => 108,
            RuntimeErr::DivideByZero => 109,
            RuntimeErr::RegisterOutOfBounds => 110,
            RuntimeErr::MaximumIterationsReached => 111,
            RuntimeErr::TooManyInputs => 112,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Simulator {
    pub memory: Vec<Instruction>,
    pub registers: Vec<i16>,
    pub program_counter: usize,
    pub counter_log: Vec<usize>,
    pub just_updated_pc: bool,
    pub debug: bool,
    pub current_regs: Vec<u8>,
    pub headless: bool,
    pub inputs: Vec<i16>,
    pub outputs: Vec<i16>,
}

impl Simulator {
    pub fn new(compiled_text: Vec<Instruction>) -> Self {
        let data_left = 256 - compiled_text.len();
        let mut memory: Vec<Instruction> = compiled_text;
        let data = Instruction::new_blank_data();

        for _ in 0..data_left {
            memory.push(data.clone());
        }

        let mut registers: Vec<i16> = Vec::new();
        for _ in 0..16 {
            registers.push(0 as i16);
        }
        Simulator {
            memory: memory,
            registers: registers,
            program_counter: 0,
            counter_log: Vec::new(),
            just_updated_pc: false,
            debug: false,
            current_regs: vec![0, 0, 0],
            headless: false,
            inputs: Vec::new(),
            outputs: Vec::new(),
        }
    }

    pub fn new_headless(compiled_text: Vec<Instruction>) -> Self {
        let mut sim = Simulator::new(compiled_text);
        sim.headless = true;
        sim
    }

    /// Function to compile a vec of HMMM instructions into
    /// a Vec of Instruction structs
    pub fn compile_hmmm(
        uncompiled_text: Vec<String>,
        is_headless: bool,
    ) -> Result<Vec<Instruction>, CompileErr> {
        let mut line_counter = 0;
        let mut compiled_text: Vec<Instruction> = Vec::new();

        for (index, line) in uncompiled_text.iter().enumerate() {
            if !(line.trim().starts_with("#")) && line.len() > 2 {
                let mut line_parts: Vec<String> = line
                    .split(&[',', ' ', '\t'][..])
                    .map(|a| String::from(a))
                    .collect();
                let line_number = line_parts.get(0).unwrap().trim().parse::<i128>();
                let comment_part = line_parts.iter().position(|a| a.starts_with("#"));

                if comment_part.is_some() {
                    line_parts.drain(comment_part.unwrap()..);
                }

                let line_parts: Vec<String> = String::from(line_parts.join(" ").trim())
                    .split_whitespace()
                    .map(|a| String::from(a))
                    .collect();

                let cleaned_line = String::from(line_parts[1..].join(" ")).to_lowercase();
                if line_number.is_err() {
                    if !is_headless {
                        raise_compile_error(
                            index,
                            CompileErr::LineNumberNotPresent,
                            line,
                            line_parts,
                        );
                    }
                    return Err(CompileErr::LineNumberNotPresent);
                } else {
                    if line_number.unwrap() != line_counter {
                        if !is_headless {
                            raise_compile_error(
                                index,
                                CompileErr::InvalidLineNumber,
                                line,
                                line_parts,
                            );
                        }
                        return Err(CompileErr::InvalidLineNumber);
                    } else {
                        let next_instruction = Instruction::new_from_text(cleaned_line.as_str());
                        if next_instruction.is_err() {
                            let err = next_instruction.unwrap_err();
                            if !is_headless {
                                raise_compile_error(index, err.clone(), line, line_parts);
                            }
                            return Err(err);
                        } else {
                            compiled_text.push(next_instruction.unwrap());
                            line_counter += 1;
                        }
                    }
                }
            }
        }

        Ok(compiled_text)
    }

    pub fn write_reg(&mut self, register: u8, data: i16) -> Result<(), RuntimeErr> {
        if register > 15 {
            return Err(RuntimeErr::InvalidRegisterLocation);
        } else if register > 0 {
            self.registers[register as usize] = data;
        }

        Ok(())
    }

    pub fn read_reg(&mut self, register: u8) -> Result<i16, RuntimeErr> {
        if register == 0 {
            return Ok(0 as i16);
        } else if register > 15 {
            return Err(RuntimeErr::InvalidRegisterLocation);
        } else {
            Ok(self.registers[register as usize])
        }
    }

    pub fn write_mem(&mut self, memory: u8, data: i16) -> Result<(), RuntimeErr> {
        let data_binary = format!("{:016b}", data);

        let data = Instruction::new_data(data_binary.as_str());

        self.memory[memory as usize] = data;
        Ok(())
    }

    pub fn read_mem(&mut self, memory: u8) -> Result<i16, RuntimeErr> {
        let data = self.memory[memory as usize].clone();
        if data.instruction_type.names[0] != "data" {
            return Err(RuntimeErr::MemoryLocationNotData);
        } else {
            let binary = data.binary_contents.join("");
            let num = i16::from_str_radix(binary.as_str(), 2);

            if num.is_err() {
                return Err(RuntimeErr::InvalidMemoryData);
            } else {
                Ok(num.unwrap())
            }
        }
    }

    /// Updates the program counter, which points to a "memory address"
    /// between 0 and 255.
    /// Logs each change for debugging purposes.
    pub fn update_pc(&mut self, new_pc: usize) -> Result<(), RuntimeErr> {
        if new_pc > 255 {
            return Err(RuntimeErr::InvalidProgramCounter);
        } else {
            self.counter_log.push(self.program_counter);
            self.program_counter = new_pc;
            Ok(())
        }
    }

    /// Sets the state of debug mode
    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }

    // Get debug state
    pub fn is_debug(&self) -> bool {
        self.debug
    }

    // Get headless state
    pub fn is_headless(&self) -> bool {
        self.headless
    }

    // Add to output
    pub fn add_output(&mut self, output: i16) {
        self.outputs.push(output);
    }

    // Set inputs
    pub fn set_inputs(&mut self, inputs: Vec<i16>) {
        self.inputs = inputs;
    }

    // Get the next input, and pop it
    pub fn get_next_input(&mut self) -> Option<i16> {
        if self.inputs.is_empty() {
            return None;
        }
        Some(self.inputs.remove(0))
    }

    // Return output vec
    pub fn get_outputs(&mut self) -> Vec<i16> {
        self.outputs.clone()
    }

    /// Function to both execute instruction on program counter
    /// and increment program counter
    pub fn step(&mut self) -> Result<(), RuntimeErr> {
        // Run memory at program counter
        let execution_result = self.execute_next();

        // If the execution resulted in an error, return it
        if execution_result.is_err() {
            return Err(execution_result.unwrap_err());
        }

        // Otherwise, increase the program counter by one if instruction
        // didn't already do thats
        if self.just_updated_pc == false {
            let update_program_counter = self.update_pc(self.program_counter + 1);

            // If there's an error (went past the final memory address), return it
            if update_program_counter.is_err() {
                return Err(update_program_counter.unwrap_err());
            }
        }

        Ok(())
    }

    /// Matches instruction name to appropriate function and executes it.
    ///
    /// Modifies self in order to change the state of memory and registers.
    ///
    /// Returns a result of either Ok or a RuntimeErr.
    pub fn execute_next(&mut self) -> Result<(), RuntimeErr> {
        // Clone the current program counter for use in instructions
        let pc = self.get_program_counter();

        if pc > 255 {
            return Err(RuntimeErr::InvalidProgramCounter);
        }

        // Make sure to rest just_updated_pc to false
        self.just_updated_pc = false;
        // Clone the current instruction from memory
        let instruction_to_run = self.get_memory(self.get_program_counter()).unwrap();
        // Get the name of the instruction for quick reference
        let instruction_name = instruction_to_run.instruction_type.names[0];

        self.current_regs = self.quick_access_regs(instruction_to_run);

        let result: Result<(), RuntimeErr> = match instruction_name {
            "data" => self.perform_data(),
            "halt" => self.perform_halt(),
            "nop" => self.perform_nop(),
            "read" => self.perform_read(),
            "write" => self.perform_write(),
            "setn" => self.perform_setn(),
            "loadr" => self.perform_loadr(),
            "storer" => self.perform_storer(),
            "popr" => self.perform_popr(),
            "pushr" => self.perform_pushr(),
            "loadn" => self.perform_loadn(),
            "storen" => self.perform_storen(),
            "addn" => self.perform_addn(),
            "copy" => self.perform_copy(),
            "neg" => self.perform_neg(),
            "add" | "sub" | "mul" | "div" | "mod" => self.perform_arithmetic(instruction_name),
            "jumpr" => self.perform_jumpr(),
            "jumpn" => self.perform_jumpn(),
            "jeqzn" => self.perform_jeqzn(),
            "jnezn" => self.perform_jnezn(),
            "jgtzn" => self.perform_jgtzn(),
            "jltzn" => self.perform_jltzn(),
            "calln" => self.perform_calln(),
            _ => Err(RuntimeErr::InvalidInstructionType),
        };

        return result;
    }

    /// Returns the current program counter as usize
    pub fn get_program_counter(&self) -> usize {
        return self.program_counter;
    }

    /// Returns the register value at the given register index
    pub fn get_register(&self, address: usize) -> Option<i16> {
        let option = self.registers.get(address);

        if option.is_none() {
            return None;
        } else {
            return Some(option.unwrap().clone());
        }
    }
    /// Returns the Instruction struct at memory[address] as Option
    pub fn get_memory(&self, address: usize) -> Option<Instruction> {
        let option = self.memory.get(address);
        if option.is_none() {
            return None;
        } else {
            return Some(option.unwrap().clone());
        }
    }
    /// Returns current counter log of program counter
    pub fn get_counter_log(&self) -> Vec<usize> {
        return self.counter_log.clone();
    }

    /// Returns the current instruction register values    
    pub fn quick_access_regs(&self, instruction_to_run: Instruction) -> Vec<u8> {
        // All binary of length 4 can be coerced into u8, and having all three
        // arguments available as numbers can be useful for instructions
        let reg_x = u8::from_str_radix(instruction_to_run.binary_contents[1].as_str(), 2).unwrap();
        let reg_y = u8::from_str_radix(instruction_to_run.binary_contents[2].as_str(), 2).unwrap();
        let reg_z = u8::from_str_radix(instruction_to_run.binary_contents[3].as_str(), 2).unwrap();

        return vec![reg_x, reg_y, reg_z];
    }

    // Get last data as i8
    pub fn get_ending_data(&self) -> Result<i8, RuntimeErr> {
        let instruction_to_run = self.get_memory(self.get_program_counter()).unwrap();

        return signed_binary_conversion(
            self.get_memory(self.get_program_counter())
                .unwrap()
                .binary_contents[2..]
                .join("")
                .as_str(),
        );
    }

    // Below are the functions for each instruction

    pub fn perform_data(&mut self) -> Result<(), RuntimeErr> {
        Err(RuntimeErr::InstructionIsData)
    }

    pub fn perform_halt(&mut self) -> Result<(), RuntimeErr> {
        Err(RuntimeErr::Halt)
    }

    pub fn perform_nop(&mut self) -> Result<(), RuntimeErr> {
        Ok(())
    }

    pub fn perform_read(&mut self) -> Result<(), RuntimeErr> {
        if self.is_headless() {
            let next_number = self.get_next_input();
            if next_number.is_none() {
                return Err(RuntimeErr::TooManyInputs);
            } else {
                return self.write_reg(self.current_regs[0], next_number.unwrap());
            }
        } else {
            loop {
                let mut line = String::new();
                if self.is_debug() {
                    let w = terminal::stdout();
                    let _ = w.act(Action::ShowCursor);
                    let _ = w.act(Action::EnableBlinking);
                    let _ = w.act(Action::MoveCursorTo(0, 29)).unwrap();
                    print!("{}", "Enter number:".on_yellow().black());
                    let _ = w.act(Action::MoveCursorTo(0, 30)).unwrap();
                    print!("                                 ");
                    let _ = w.act(Action::MoveCursorTo(0, 30)).unwrap();
                    stdin().lock().read_line(&mut line).unwrap();
                    let _ = w.act(Action::DisableBlinking);
                    let _ = w.act(Action::HideCursor);
                } else {
                    println!("{}", "Enter number:".on_yellow().black());
                    io::stdin().read_line(&mut line).unwrap();
                }
                line = line.trim().to_string();
                if line == "q" {
                    return Err(RuntimeErr::Halt);
                }
                let number = line.parse::<i16>();
                if number.is_ok() {
                    if self.is_debug() {
                        let w = terminal::stdout();
                        w.act(Action::MoveCursorTo(16, 29)).unwrap();
                        print!("                                        ");
                    }
                    return self.write_reg(self.current_regs[0], number.unwrap());
                }
                if self.is_debug() {
                    let w = terminal::stdout();
                    w.act(Action::MoveCursorTo(16, 29)).unwrap();
                    print!("Invalid number! Please try again...");
                } else {
                    println!("Invalid number! Please try again...");
                }
            }
        }
    }

    pub fn perform_write(&mut self) -> Result<(), RuntimeErr> {
        if self.is_headless() {
            let read_num = self.read_reg(self.current_regs[0])?;
            self.add_output(read_num);
        } else {
            if self.is_debug() {
                let w = terminal::stdout();
                w.act(Action::MoveCursorTo(50, 8)).unwrap();
                let to_print = format!("{:<10}", self.read_reg(self.current_regs[0])?);
                print!("{}", to_print);
            } else {
                println!(
                    "{}\n{}",
                    "HMMM OUT:".on_green().black(),
                    self.read_reg(self.current_regs[0])?
                );
            }
        }
        return Ok(());
    }

    pub fn perform_setn(&mut self) -> Result<(), RuntimeErr> {
        return self.write_reg(self.current_regs[0], self.get_ending_data()? as i16);
    }

    pub fn perform_loadr(&mut self) -> Result<(), RuntimeErr> {
        let index = self.read_reg(self.current_regs[1]);

        if index.is_err() {
            return Err(index.unwrap_err());
        }

        let index = index.unwrap();

        if index > 255 || index < 0 {
            return Err(RuntimeErr::InvalidMemoryLocation);
        }

        let data = self.read_mem(index as u8);

        if data.is_err() {
            return Err(data.unwrap_err());
        }

        return self.write_reg(self.current_regs[0], data.unwrap());
    }

    pub fn perform_storer(&mut self) -> Result<(), RuntimeErr> {
        let index = self.read_reg(self.current_regs[1]);
        if index.is_err() {
            return Err(index.unwrap_err());
        }

        let index = index.unwrap();

        if index > 255 || index < 0 {
            return Err(RuntimeErr::InvalidMemoryLocation);
        }

        let data = self.read_reg(self.current_regs[1] as u8)?;

        return self.write_mem(index as u8, data);
    }

    pub fn perform_popr(&mut self) -> Result<(), RuntimeErr> {
        let reg_y_data = self.read_reg(self.current_regs[1])?;

        let reg_y_data = reg_y_data;

        if reg_y_data > 255 || reg_y_data < 0 {
            return Err(RuntimeErr::InvalidMemoryLocation);
        }

        let change_reg = self.write_reg(self.current_regs[1], reg_y_data - 1)?;

        let reg_y_data = reg_y_data as u8;

        let mem_data = self.read_mem(reg_y_data - 1)?;

        self.write_reg(self.current_regs[0], mem_data)
    }

    pub fn perform_pushr(&mut self) -> Result<(), RuntimeErr> {
        let reg_y_data = self.read_reg(self.current_regs[1])?;

        if reg_y_data > 255 || reg_y_data < 0 {
            return Err(RuntimeErr::InvalidMemoryData);
        }

        let data = self.read_reg(self.current_regs[0])?;

        let mem_write = self.write_mem(reg_y_data as u8, data);

        return self.write_reg(self.current_regs[1], reg_y_data + 1);
    }

    pub fn perform_loadn(&mut self) -> Result<(), RuntimeErr> {
        let ending_data = self.get_ending_data()?;

        let memory_data = self.read_mem(ending_data as u8)?;

        return self.write_reg(self.current_regs[0], memory_data);
    }

    pub fn perform_storen(&mut self) -> Result<(), RuntimeErr> {
        let ending_data = self.get_ending_data()?;
        let reg_x_data = self.read_reg(self.current_regs[0])?;

        return self.write_mem(ending_data as u8, reg_x_data);
    }

    pub fn perform_addn(&mut self) -> Result<(), RuntimeErr> {
        let ending_data = self.get_ending_data()?;

        let reg_x_data = self.read_reg(self.current_regs[0])?;

        return self.write_reg(self.current_regs[0], reg_x_data + ending_data as i16);
    }

    pub fn perform_copy(&mut self) -> Result<(), RuntimeErr> {
        let reg_y_data = self.read_reg(self.current_regs[1])?;

        return self.write_reg(self.current_regs[0], reg_y_data);
    }

    pub fn perform_neg(&mut self) -> Result<(), RuntimeErr> {
        let reg_y_data = self.read_reg(self.current_regs[1])?;

        return self.write_reg(self.current_regs[0], -reg_y_data);
    }

    pub fn perform_arithmetic(&mut self, name: &str) -> Result<(), RuntimeErr> {
        let reg_z_data = self.read_reg(self.current_regs[2])?;

        let reg_y_data = self.read_reg(self.current_regs[1])?;

        if reg_z_data == 0 && name == "div" {
            return Err(RuntimeErr::DivideByZero);
        }
        // Coerce to a higher level data type
        // so that we can detect an out of bounds error
        let result: i32 = match name {
            "add" => reg_y_data as i32 + reg_z_data as i32,
            "sub" => reg_y_data as i32 - reg_z_data as i32,
            "mul" => reg_y_data as i32 * reg_z_data as i32,
            "div" => reg_y_data as i32 / reg_z_data as i32,
            "mod" => reg_y_data as i32 % reg_z_data as i32,
            _ => 0,
        };

        if result > i16::MAX as i32 || result < i16::MIN as i32 {
            return Err(RuntimeErr::RegisterOutOfBounds);
        }

        return self.write_reg(self.current_regs[0], result as i16);
    }

    pub fn perform_jumpr(&mut self) -> Result<(), RuntimeErr> {
        let reg_x_data = self.read_reg(self.current_regs[0])?;
        if reg_x_data < 0 {
            Err(RuntimeErr::InvalidProgramCounter)
        } else {
            self.just_updated_pc = true;
            return self.update_pc(reg_x_data as usize);
        }
    }

    pub fn perform_jumpn(&mut self) -> Result<(), RuntimeErr> {
        self.just_updated_pc = true;
        return self.update_pc(self.get_ending_data()? as usize);
    }

    pub fn perform_jeqzn(&mut self) -> Result<(), RuntimeErr> {
        let reg_x_data = self.read_reg(self.current_regs[0])?;
        if reg_x_data == 0 {
            self.just_updated_pc = true;
            self.update_pc(self.get_ending_data()? as usize)
        } else {
            Ok(())
        }
    }

    pub fn perform_jnezn(&mut self) -> Result<(), RuntimeErr> {
        let reg_x_data = self.read_reg(self.current_regs[0])?;
        if reg_x_data != 0 {
            self.just_updated_pc = true;
            self.update_pc(self.get_ending_data()? as usize)
        } else {
            Ok(())
        }
    }

    pub fn perform_jgtzn(&mut self) -> Result<(), RuntimeErr> {
        let reg_x_data = self.read_reg(self.current_regs[0])?;
        if reg_x_data > 0 {
            self.just_updated_pc = true;
            self.update_pc(self.get_ending_data()? as usize)
        } else {
            Ok(())
        }
    }

    pub fn perform_jltzn(&mut self) -> Result<(), RuntimeErr> {
        let reg_x_data = self.read_reg(self.current_regs[0])?;
        if reg_x_data < 0 {
            self.just_updated_pc = true;
            self.update_pc(self.get_ending_data()? as usize)
        } else {
            Ok(())
        }
    }

    pub fn perform_calln(&mut self) -> Result<(), RuntimeErr> {
        let update_rg = self.write_reg(
            self.current_regs[0],
            (self.get_program_counter() + 1) as i16,
        )?;

        self.just_updated_pc = true;

        return self.update_pc(self.get_ending_data()? as usize);
    }
}

pub fn signed_binary_conversion(binary: &str) -> Result<i8, RuntimeErr> {
    let is_negative: bool = { binary.starts_with("1") };
    let mut binary_mut: String = binary.to_owned();
    if is_negative {
        binary_mut = binary.chars().rev().collect();
        let mut found_1 = false;
        let mut temp_string: String = "".to_string();

        for i in binary_mut.chars() {
            if found_1 == true {
                if i == '1' {
                    temp_string += "0";
                } else {
                    temp_string += "0";
                }
            } else if i == '1' && found_1 == false {
                found_1 = true;
                temp_string += "1";
            }
        }

        binary_mut = temp_string.chars().rev().collect();
    }

    let decoded_signed = i8::from_str_radix(binary_mut.as_str(), 2);

    if decoded_signed.is_err() {
        return Err(RuntimeErr::InvalidSignedNumber);
    }

    if is_negative {
        return Ok(0 - decoded_signed.unwrap());
    } else {
        return Ok(decoded_signed.unwrap());
    }
}

pub fn split_binary_to_chunks(text: String) -> String {
    text.chars()
        .enumerate()
        .flat_map(|(i, c)| {
            if i != 0 && i % 4 == 0 {
                Some(' ')
            } else {
                None
            }
            .into_iter()
            .chain(std::iter::once(c))
        })
        .collect::<String>()
}
