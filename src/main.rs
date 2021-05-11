use clap::{App, Arg};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::*;

use lazy_static::lazy_static;
use std::*;
use terminal::*;

static UNCOMPILED: &str = ".hmmm";
static COMPILED: &str = ".hb";

lazy_static! {
    static ref INSTRUCTION_LOOKUP: Vec<InstructionType> = vec![
        InstructionType::new(
            vec!["halt"],
            "0000 0000 0000 0000",
            "1111 1111 1111 1111",
            ""
        ),
        InstructionType::new(
            vec!["read"],
            "0000 0000 0000 0001",
            "1111 0000 1111 1111",
            "r"
        ),
        InstructionType::new(
            vec!["write"],
            "0000 0000 0000 0010",
            "1111 0000 1111 1111",
            "r"
        ),
        InstructionType::new(
            vec!["jumpr", "jump"],
            "0000 0000 0000 0011",
            "1111 0000 1111 1111",
            "r"
        ),
        InstructionType::new(
            vec!["setn"],
            "0001 0000 0000 0000",
            "1111 0000 0000 0000",
            "rs"
        ),
        InstructionType::new(
            vec!["loadn"],
            "0010 0000 0000 0000",
            "1111 0000 0000 0000",
            "ru"
        ),
        InstructionType::new(
            vec!["storen"],
            "0011 0000 0000 0000",
            "1111 0000 0000 0000",
            "ru"
        ),
        InstructionType::new(
            vec!["loadr"],
            "0100 0000 0000 0000",
            "1111 0000 0000 0000",
            ""
        ),
        InstructionType::new(
            vec!["storer"],
            "0100 0000 0000 0001",
            "1111 0000 0000 0000",
            "rr"
        ),
        InstructionType::new(
            vec!["popr"],
            "0100 0000 0000 0010",
            "1111 0000 0000 1111",
            "rr"
        ),
        InstructionType::new(
            vec!["pushr"],
            "0100 0000 0000 0011",
            "1111 0000 0000 1111",
            "rr"
        ),
        InstructionType::new(
            vec!["addn"],
            "0101 0000 0000 0000",
            "1111 0000 0000 0000",
            "rs"
        ),
        InstructionType::new(
            vec!["nop"],
            "0110 0000 0000 0000",
            "1111 1111 1111 1111",
            ""
        ),
        InstructionType::new(
            vec!["copy", "mov"],
            "0110 0000 0000 0000",
            "1111 0000 0000 1111",
            "rr"
        ),
        InstructionType::new(
            vec!["add"],
            "0110 0000 0000 0000",
            "1111 0000 0000 0000",
            "rrr"
        ),
        InstructionType::new(
            vec!["neg"],
            "0111 0000 0000 0000",
            "1111 0000 1111 0000",
            "rzr"
        ),
        InstructionType::new(
            vec!["sub"],
            "0111 0000 0000 0000",
            "1111 0000 0000 0000",
            "rrr"
        ),
        InstructionType::new(
            vec!["mul"],
            "1000 0000 0000 0000",
            "1111 0000 0000 0000",
            "rrr"
        ),
        InstructionType::new(
            vec!["div"],
            "1001 0000 0000 0000",
            "1111 0000 0000 0000",
            "rrr"
        ),
        InstructionType::new(
            vec!["mod"],
            "1010 0000 0000 0000",
            "1111 0000 0000 0000",
            "rrr"
        ),
        InstructionType::new(
            vec!["jumpn"],
            "1011 0000 0000 0000",
            "1111 1111 0000 0000",
            "zu"
        ),
        InstructionType::new(
            vec!["calln", "call"],
            "1011 0000 0000 0000",
            "1111 0000 0000 0000",
            "ru"
        ),
        InstructionType::new(
            vec!["jeqzn", "jeqz"],
            "1100 0000 0000 0000",
            "1111 0000 0000 0000",
            "ru"
        ),
        InstructionType::new(
            vec!["jnezn", "jnez"],
            "1101 0000 0000 0000",
            "1111 0000 0000 0000",
            "ru"
        ),
        InstructionType::new(
            vec!["jgtzn", "jgtz"],
            "1110 0000 0000 0000",
            "1111 0000 0000 0000",
            "ru"
        ),
        InstructionType::new(
            vec!["jltzn", "jltz"],
            "1111 0000 0000 0000",
            "1111 0000 0000 0000",
            "ru"
        ),
        InstructionType::new(
            vec!["data"],
            "0000 0000 0000 0000",
            "0000 0000 0000 0000",
            "n"
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
    names: Vec<&'static str>,
    /// String that will match an instruction from
    /// a .hmmm file
    match_string: &'static str,
    /// String that will match where additional information
    /// such as numbers or registers lay
    mask_string: &'static str,
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
    arguments: &'static str,
}

impl InstructionType {
    pub fn new(
        names: Vec<&'static str>,
        match_string: &'static str,
        mask_string: &'static str,
        arguments: &'static str,
    ) -> InstructionType {
        InstructionType {
            names: names,
            match_string: match_string,
            mask_string: mask_string,
            arguments: arguments,
        }
    }
}

#[derive(Debug)]
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

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Instruction {
    instruction_type: InstructionType,
    text_contents: String,
    binary_contents: Vec<String>,
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
                text_contents: String::from(instruction_type.clone().names[0]),
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
                let combined_binary = format!(
                    "{}{}",
                    binary_contents[slots_filled],
                    binary_contents[slots_filled + 1]
                );
                instruction_args.push(format!(
                    "{}",
                    i32::from_str_radix(combined_binary.as_str(), 2).unwrap()
                ));
                slots_filled += 3;
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
}

#[derive(Debug, PartialEq)]
pub enum RuntimeErr {
    InvalidRegisterLocation,
    MemoryLocationNotData,
    InvalidMemoryData,
    InvalidMemoryLocation,
    InvalidData,
    Halt,
    InvalidProgramCounter,
    InstructionIsData,
    InvalidInstructionType,
    DivideByZero,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Simulator {
    pub memory: Vec<Instruction>,
    pub registers: Vec<i16>,
    pub program_counter: usize,
    pub counter_log: Vec<usize>,
}

impl Simulator {
    pub fn new(compiled_text: Vec<Instruction>) -> Self {
        let data_left = 256 - compiled_text.len();
        let mut memory: Vec<Instruction> = compiled_text;
        let data = Instruction::new_from_binary("0000 0000 0000 0000").unwrap();

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
        }
    }

    pub fn write_rg(&mut self, register: u8, data: i16) -> Result<(), RuntimeErr> {
        if register > 15 {
            return Err(RuntimeErr::InvalidRegisterLocation);
        } else if register > 0 {
            self.registers[register as usize] = data;
        }

        Ok(())
    }

    pub fn read_rg(&mut self, register: u8) -> Result<i16, RuntimeErr> {
        if register == 0 {
            return Ok(0 as i16);
        } else if register > 15 {
            return Err(RuntimeErr::InvalidRegisterLocation);
        } else {
            Ok(self.registers[register as usize])
        }
    }

    pub fn write_mem(&mut self, memory: u8, data: i16) -> Result<(), RuntimeErr> {
        let data_binary = split_binary_to_chunks(format!("0000{:016b}", data));
        let data = Instruction::new_from_binary(data_binary.as_str());
        if data.is_err() {
            return Err(RuntimeErr::InvalidData);
        } else {
            self.memory[memory as usize] = data.unwrap();
            Ok(())
        }
    }

    pub fn read_mem(&mut self, memory: u8) -> Result<i16, RuntimeErr> {
        let data = self.memory[memory as usize].clone();

        if data.instruction_type.names[0] != "data" {
            return Err(RuntimeErr::MemoryLocationNotData);
        } else {
            let binary = data.binary_contents[1..].join("");
            let num = i16::from_str_radix(binary.as_str(), 2);

            if num.is_err() {
                return Err(RuntimeErr::InvalidMemoryData);
            } else {
                Ok(num.unwrap())
            }
        }
    }

    pub fn update_pc(&mut self, new_pc: usize) -> Result<(), RuntimeErr> {
        // Errors on 256 not 255 due to update_pc being called at the start
        // of the step() function
        if new_pc > 256 {
            return Err(RuntimeErr::InvalidProgramCounter);
        } else {
            self.counter_log.push(self.program_counter);
            self.program_counter = new_pc;
            Ok(())
        }
    }

    /// Massive single function to step though a line of instructions
    ///
    /// Modifies self in order to change the state of memory and registers
    ///
    /// Returns a result of either Ok or a RuntimeErr
    pub fn step(&mut self) -> Result<(), RuntimeErr> {
        // Clone the current program counter for use in instructions
        let pc = self.program_counter.clone();
        // Clone the current instruction from memory
        let instruction_to_run = self.memory[pc].clone();
        // Get the name of the instruction for quick reference
        let instruction_name = instruction_to_run.instruction_type.names[0];

        // Update the program counter
        let update_pc = self.update_pc(pc + 1);
        // Will error if the program counter is invalid
        if update_pc.is_err() {
            return Err(update_pc.unwrap_err());
        }

        // All binary of length 4 can be coerced into u8, and having all three
        // arguments available as numbers can be useful for instructions
        let reg_x = u8::from_str_radix(instruction_to_run.binary_contents[1].as_str(), 2).unwrap();
        let reg_y = u8::from_str_radix(instruction_to_run.binary_contents[2].as_str(), 2).unwrap();
        let reg_z = u8::from_str_radix(instruction_to_run.binary_contents[3].as_str(), 2).unwrap();

        // The last two binary sections are used as data often, so we can get that as a number
        let ending_data_u8 =
            u8::from_str_radix(instruction_to_run.binary_contents[2..].join("").as_str(), 2)
                .unwrap();
        // Same can be said for the data in regX
        let reg_x_data = self.read_rg(reg_x);

        if reg_x_data.is_err() {
            return Err(reg_x_data.unwrap_err());
        }

        let reg_x_data = reg_x_data.unwrap();

        if instruction_name == "data" {
            // You cannot execute data
            return Err(RuntimeErr::InstructionIsData);
        } else if instruction_name == "halt" {
            // Exit if halting
            return Err(RuntimeErr::Halt);
        } else if instruction_name == "nop" {
            // Do nothing, and just return an Ok
            return Ok(());
        } else if instruction_name == "read" {
            loop {
                let mut line = String::new();
                println!("Enter number:");
                io::stdin().read_line(&mut line).unwrap();
                line = line.trim().to_string();

                if line == "q" {
                    return Err(RuntimeErr::Halt);
                }

                let number = line.parse::<i16>();

                if number.is_ok() {
                    return self.write_rg(reg_x, number.unwrap());
                }

                println!("Invalid number! Please try again...");
            }
        } else if instruction_name == "write" {
            println!("{}", reg_x_data);
            return Ok(());
        } else if instruction_name == "setn" {
            let ending_data_i8 =
                i8::from_str_radix(instruction_to_run.binary_contents[2..].join("").as_str(), 2);

            if ending_data_i8.is_err() {
                return Err(RuntimeErr::InvalidData);
            }

            return self.write_rg(reg_x, ending_data_i8.unwrap() as i16);
        } else if instruction_name == "loadr" {
            let data = self.read_mem(reg_y);

            if data.is_err() {
                return Err(data.unwrap_err());
            }

            return self.write_rg(reg_x, data.unwrap());
        } else if instruction_name == "storer" {
            return self.write_mem(reg_y, reg_x_data);
        } else if instruction_name == "popr" {
            let reg_y_data = self.read_rg(reg_y);

            if reg_y_data.is_err() {
                return Err(reg_y_data.unwrap_err());
            }

            let reg_y_data = reg_y_data.unwrap();

            if reg_y_data > 255 || reg_y_data < 0 {
                return Err(RuntimeErr::InvalidMemoryLocation);
            }

            let change_reg = self.write_rg(reg_y, reg_y_data - 1);

            if change_reg.is_err() {
                return Err(change_reg.unwrap_err());
            }

            let reg_y_data = reg_y_data as u8;

            let mem_data = self.read_mem(reg_y_data - 1);

            if mem_data.is_err() {
                return Err(mem_data.unwrap_err());
            }

            let mem_data = mem_data.unwrap();

            return self.write_rg(reg_x, mem_data);
        } else if instruction_name == "pushr" {
            let reg_y_data = self.read_rg(reg_y);
            if reg_y_data.is_err() {
                return Err(reg_y_data.unwrap_err());
            }

            let reg_y_data = reg_y_data.unwrap();

            if reg_y_data > 255 || reg_y_data < 0 {
                return Err(RuntimeErr::InvalidMemoryData);
            }

            let mem_write = self.write_mem(reg_y_data as u8, reg_x_data);

            if mem_write.is_err() {
                return Err(mem_write.unwrap_err())
            }

            return self.write_rg(reg_y, reg_y_data + 1);
        } else if instruction_name == "loadn" {
            let memory_data = self.read_mem(ending_data_u8);

            if memory_data.is_err() {
                return Err(memory_data.unwrap_err());
            }

            let memory_data = memory_data.unwrap();

            return self.write_rg(reg_x, memory_data);
        } else if instruction_name == "storen" {
            return self.write_mem(ending_data_u8, reg_x_data);
        } else if instruction_name == "addn" {
            let ending_data_i8 =
                i8::from_str_radix(instruction_to_run.binary_contents[2..].join("").as_str(), 2);

            if ending_data_i8.is_err() {
                return Err(RuntimeErr::InvalidData);
            }
            return self.write_rg(reg_x, reg_x_data + ending_data_i8.unwrap() as i16);
        } else if instruction_name == "copy" {
            let reg_y_data = self.read_rg(reg_y);

            if reg_y_data.is_err() {
                return Err(reg_y_data.unwrap_err());
            }

            let reg_y_data = reg_y_data.unwrap();

            return self.write_rg(reg_x, reg_y_data);
        } else if instruction_name == "neg" {
            let reg_y_data = self.read_rg(reg_y);

            if reg_y_data.is_err() {
                return Err(reg_y_data.unwrap_err());
            }

            let reg_y_data = reg_y_data.unwrap();

            return self.write_rg(reg_x, -reg_y_data);
        } else if vec!["add", "sub", "mul", "div", "mod"].contains(&instruction_name) {
            let reg_z_data = self.read_rg(reg_z);

            if reg_z_data.is_err() {
                return Err(reg_z_data.unwrap_err());
            }

            let reg_z_data = reg_z_data.unwrap();

            let reg_y_data = self.read_rg(reg_y);

            if reg_y_data.is_err() {
                return Err(reg_y_data.unwrap_err());
            }

            let reg_y_data = reg_y_data.unwrap();

            let result: i16;

            if reg_z_data == 0 && instruction_name == "div" {
                return Err(RuntimeErr::DivideByZero);
            }
            match instruction_name {
                "add" => result = reg_y_data + reg_z_data,
                "sub" => result = reg_y_data - reg_z_data,
                "mul" => result = reg_y_data * reg_z_data,
                "div" => result = reg_y_data / reg_z_data,
                "mod" => result = reg_y_data % reg_z_data,
                _ => result = 0,
            }

            return self.write_rg(reg_x, result);
        } else if instruction_name == "jumpr" {
            if reg_x_data < 0 {
                return Err(RuntimeErr::InvalidProgramCounter);
            }

            return self.update_pc(reg_x_data as usize);
        } else if instruction_name == "jumpn" {
            return self.update_pc(ending_data_u8 as usize);
        } else if instruction_name == "jeqzn" {
            if reg_x_data == 0 {
                return self.update_pc(ending_data_u8 as usize);
            } else {
                Ok(())
            }
        } else if instruction_name == "jnezn" {
            if reg_x_data != 0 {
                return self.update_pc(ending_data_u8 as usize);
            } else {
                Ok(())
            }
        } else if instruction_name == "jgtzn" {
            if reg_x_data > 0 {
                return self.update_pc(ending_data_u8 as usize);
            } else {
                Ok(())
            }
        } else if instruction_name == "jltzn" {
            if reg_x_data < 0 {
                return self.update_pc(ending_data_u8 as usize);
            } else {
                Ok(())
            }
        } else if instruction_name == "calln" {
            let update_rg = self.write_rg(reg_x, (pc + 1) as i16);

            if update_rg.is_err() {
                return Err(update_rg.unwrap_err());
            }

            return self.update_pc(ending_data_u8 as usize);
        } else {
            // This should never happen. But just in case...
            Err(RuntimeErr::InvalidInstructionType)
            // Like, there is no way this should ever happen
            // unless you modify the code. Just sayin'
        }
    }
}

fn split_binary_to_chunks(text: String) -> String {
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

/// Funcition to load any text file as a Vec of Strings
fn load_file(path: &str) -> std::io::Result<Vec<String>> {
    let reader = BufReader::new(File::open(path).expect("Cannot open file.hmmm"));
    let mut output_vec: Vec<String> = Vec::new();
    for line in reader.lines() {
        output_vec.push(line?);
    }

    Ok(output_vec)
}

/// Function to pretty-print a compilation error and exit
/// the program gracefully
fn raise_compile_error(
    line_num: usize,
    error: CompileErr,
    raw_line: &String,
    line_parts: Vec<String>,
) {
    let args: String = line_parts[2..].join(" ");
    println!("▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀");
    println!("████    COMPILATION UNSUCCESSFUL    ████");
    println!("▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄");
    println!("\nERROR ON LINE {}: {:?}", line_num, error);
    println!("Raw: \"{}\"\n", raw_line);
    println!("▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀");
    println!("█           Interpreted As: ");
    println!("█ Line █ Command █ Arguments ");
    println!("█ {:4} █ {:7} █ {:15}", line_parts[0], line_parts[1], args);
    println!("▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄");
    println!("Exiting...");
    exit(1);
}

/// Function to pretty-print a runtime error and exit
/// the program gracefully
fn raise_runtime_error(sim: &Simulator, error: &RuntimeErr) {
    let last_run_line = sim.counter_log.last().unwrap().to_owned();
    println!("▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀");
    println!("████    SIMULATION  UNSUCCESSFUL    ████");
    println!("▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄");
    println!("\nERROR EXECUTING ADDRESS {}: {:?}", last_run_line, error);
    println!(
        "MEMORY ADDRESS CONTENTS: {} {}\n",
        sim.memory[last_run_line].instruction_type.names[0],
        sim.memory[last_run_line].text_contents
    );
    println!("▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀");
    println!("█             REGISTER CONTENTS             █");

    for row in 0..4 {
        println!(
            "█    R{: <2}   █    R{: <2}   █    R{: <2}   █    R{: <2}   █",
            row * 4,
            (row * 4) + 1,
            (row * 4) + 2,
            (row * 4) + 3
        );
        println!(
            "█ {:8} █ {:8} █ {:8} █ {:8} █",
            sim.registers[row * 4],
            sim.registers[(row * 4) + 1],
            sim.registers[(row * 4) + 2],
            sim.registers[(row * 4) + 3]
        );
    }
    println!("▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄");
    println!("Exiting...");
    exit(1);
}

/// Function to compile a vec of HMMM instructions into
/// a Vec of Instruction structs
fn compile_hmmm(uncompiled_text: Vec<String>) -> Vec<Instruction> {
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
                raise_compile_error(index, CompileErr::LineNumberNotPresent, line, line_parts);
            } else {
                if line_number.unwrap() != line_counter {
                    raise_compile_error(index, CompileErr::InvalidLineNumber, line, line_parts);
                } else {
                    let next_instruction = Instruction::new_from_text(cleaned_line.as_str());
                    if next_instruction.is_err() {
                        raise_compile_error(index, next_instruction.unwrap_err(), line, line_parts);
                    } else {
                        compiled_text.push(next_instruction.unwrap());
                        line_counter += 1;
                    }
                }
            }
        }
    }

    compiled_text
}

/// Function to read a vec of binary HMMM text into
/// a Vec of Instruction structs
fn read_compiled_hmmm(raw_binary: Vec<String>) -> Vec<Instruction> {
    let mut compiled_text: Vec<Instruction> = Vec::new();

    for line in raw_binary {
        let next_instruction = Instruction::new_from_binary(line.as_str());

        if next_instruction.is_err() {
            panic!("{:?}", next_instruction.err())
        }

        compiled_text.push(next_instruction.unwrap())
    }

    compiled_text
}

/// Simple function to write a program as uncompiled HMMM code
/// Useful for "decompiling" a compiled program
fn write_uncompiled_hmmm(path: &str, compiled_text: Vec<Instruction>) -> std::io::Result<()> {
    let mut contents = String::from("");

    for (index, instruction) in compiled_text.iter().enumerate() {
        contents = format!(
            "{}{} {} {}\n",
            contents, index, instruction.instruction_type.names[0], instruction.text_contents
        );
    }

    contents = String::from(contents.trim_end());

    fs::write(path, contents)?;
    Ok(())
}

/// Function to write a program as a compiled .hb binary
fn write_compiled_hmmm(path: &str, compiled_text: Vec<Instruction>) -> std::io::Result<()> {
    let mut contents = String::from("");

    for instruction in compiled_text {
        let binary = instruction.binary_contents.join(" ");
        contents = format!("{}{}\n", contents, binary);
    }

    contents = String::from(contents.trim_end());

    fs::write(path, contents)?;
    Ok(())
}

fn main() -> terminal::error::Result<()> {
    // Create the terminal object just to have an easy way
    // to clear it
    let terminal = terminal::stdout();
    terminal.act(Action::ClearTerminal(Clear::All))?;

    // Setup command line matches
    let matches = App::new("HMMM Compiler")
        .version("1.0")
        .author("Ethan Vazquez <edv121@outlook.com>")
        .about("A compiler, decompiler, debugger, and simulator for Harvey Mudd Miniature Machine (HMMM)")
        .arg(Arg::with_name("input")
                 .short("i")
                 .long("input")
                 .takes_value(true)
                 .help("Input .hmmm or .hb file"))
        .arg(Arg::with_name("output")
                 .short("o")
                 .long("output")
                 .takes_value(true)
                 .help("Output location of either .hmmm or .hb file"))
        .arg(Arg::with_name("debug")
                 .short("d")
                 .long("debug")
                 .takes_value(false)
                 .help("Use debug mode for stepping through simulator"))
        .arg(Arg::with_name("no-run")
                 .short("n")
                 .long("no-run")
                 .takes_value(false)
                 .help("Do not simulate (run) the program on compilation"))
        .get_matches();

    if matches.value_of("input").is_none() {
        println!("Error: Please specify a file to compile/run!");
        exit(1);
    } else {
        let file_path: &str = matches.value_of("input").unwrap();

        // Setup the vec for the compiled Instructions
        let compiled_text: Vec<Instruction>;

        // Check to see what type of file is being loaded
        if file_path.ends_with(UNCOMPILED) {
            // If it's uncompiled, load it
            let uncompiled_text = load_file(file_path).unwrap();

            // Then, compile it into Instruction structs
            compiled_text = compile_hmmm(uncompiled_text);
        } else if file_path.ends_with(COMPILED) {
            // If it's already compiled, load it
            let raw_binary = load_file(file_path).unwrap();

            // Then, interpret it into Instruction structs
            compiled_text = read_compiled_hmmm(raw_binary);
        } else {
            panic!("Unknown filetype!");
        }

        // If compiles without error, print out a success
        // message and the first 9 lines, with the last being
        // printed also if there are > 9 lines
        println!("▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀");
        println!("████     COMPILATION SUCCESSFUL     ████");
        println!("▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄");
        println!("\n");
        println!("▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀");
        println!("█ Line █ Command █ Arguments           █");
        println!("▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄");

        for (index, line) in compiled_text.iter().enumerate() {
            if index > 9 {
                // Print seperator to show the jump in line number
                println!("........................................");
                let last = compiled_text.last().unwrap();
                println!(
                    "█ {:4} █ {:7} █ {:19} █  >>    {}",
                    compiled_text.len() - 1,
                    last.instruction_type.names[0],
                    last.text_contents,
                    last.binary_contents.join(" ")
                );
                break;
            }
            println!(
                "█ {:4} █ {:7} █ {:19} █  >>    {}",
                index,
                line.instruction_type.names[0],
                line.text_contents,
                line.binary_contents.join(" ")
            );
        }

        println!("█▄▄▄▄▄▄█▄▄▄▄▄▄▄▄▄█▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄█\n\n");

        // Output file if given path
        if matches.value_of("output").is_some() {
            let output_file = matches.value_of("output").unwrap();
            let result;

            if output_file.ends_with(UNCOMPILED) {
                result = write_uncompiled_hmmm(output_file, compiled_text.clone());
            } else if output_file.ends_with(COMPILED) {
                result = write_compiled_hmmm(output_file, compiled_text.clone());
            } else {
                println!("No output type specified, writing as binary...");
                // If no ending, just tack on a .hb extension and write out as binary
                result = write_compiled_hmmm(
                    format!("{}.hb", output_file).as_str(),
                    compiled_text.clone(),
                );
            }

            if result.is_err() {
                println!("Error writing output file! Continuing...");
            }
        }

        // Run simulation if --no-run flag is not present
        if matches.value_of("no-run").is_none() {
            // Create it as new struct from compiled HMMM
            let mut simulator = Simulator::new(compiled_text);

            // If not in debug mode, just run in a loop
            if matches.value_of("debug").is_none() {
                loop {
                    // Attempt to run a step in the simulator
                    let result = &simulator.step();
                    // If it's an error, raise it
                    if result.is_err() {
                        let result_err = result.as_ref().unwrap_err();
                        // If the error is Halt, exit quietly, as that is the
                        // program successfully finishing
                        if result_err == &RuntimeErr::Halt {
                            println!("Program has reached end, exiting...");
                            exit(0);
                        } else {
                            // If not, raise that error!
                            raise_runtime_error(&simulator, &result_err);
                        }
                    }
                }
            }
            // If in debug mode, start debug mode
            else {
            }
        }

        Ok(())
    }
}
