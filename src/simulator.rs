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
            "rr"
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

    pub fn new_data() -> Self {
        Instruction {
            instruction_type: InstructionType::new(
                vec!["data"],
                "0000 0000 0000 0000",
                "0000 0000 0000 0000",
                "n",
            ),
            binary_contents: vec![
                "0000".to_string(),
                "0000".to_string(),
                "0000".to_string(),
                "0000".to_string(),
            ],
            text_contents: "data".to_string(),
        }
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

#[derive(Debug, PartialEq)]
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
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Simulator {
    pub memory: Vec<Instruction>,
    pub registers: Vec<i16>,
    pub program_counter: usize,
    pub counter_log: Vec<usize>,
    pub just_updated_pc: bool,
    pub debug: bool,
}

impl Simulator {
    pub fn new(compiled_text: Vec<Instruction>) -> Self {
        let data_left = 256 - compiled_text.len();
        let mut memory: Vec<Instruction> = compiled_text;
        let data = Instruction::new_data();

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

    /// Massive single function to step though a line of instructions
    ///
    /// Modifies self in order to change the state of memory and registers
    ///
    /// Returns a result of either Ok or a RuntimeErr
    pub fn execute_next(&mut self) -> Result<(), RuntimeErr> {
        // Clone the current program counter for use in instructions
        let pc = self.program_counter.clone();
        // Make sure to rest just_updated_pc to false
        self.just_updated_pc = false;
        // Clone the current instruction from memory
        let instruction_to_run = self.memory[pc].clone();
        // Get the name of the instruction for quick reference
        let instruction_name = instruction_to_run.instruction_type.names[0];

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
        let result: Result<(), RuntimeErr> = match instruction_name {
            "data" => return Err(RuntimeErr::InstructionIsData),
            "halt" => return Err(RuntimeErr::Halt),
            "nop" => return Ok(()),
            "read" => loop {
                let mut line = String::new();
                if self.debug == true {
                    let mut w = terminal::stdout();
                    w.act(Action::ShowCursor);
                    w.act(Action::EnableBlinking);
                    w.act(Action::MoveCursorTo(0, 29)).unwrap();
                    print!("{}", "Enter number:".on_yellow().black());
                    w.act(Action::MoveCursorTo(0, 30)).unwrap();
                    print!("                                 ");
                    w.act(Action::MoveCursorTo(0, 30)).unwrap();
                    stdin().lock().read_line(&mut line).unwrap();
                    w.act(Action::DisableBlinking);
                    w.act(Action::HideCursor);
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
                    if self.debug == true {
                        let w = terminal::stdout();
                        w.act(Action::MoveCursorTo(16, 29)).unwrap();
                        print!("                                        ");
                    }
                    return self.write_rg(reg_x, number.unwrap());
                }
                if self.debug == true {
                    let w = terminal::stdout();
                    w.act(Action::MoveCursorTo(16, 29)).unwrap();
                    print!("Invalid number! Please try again...");
                } else {
                    println!("Invalid number! Please try again...");
                }
            },
            "write" => {
                if self.debug == true {
                    let w = terminal::stdout();
                    w.act(Action::MoveCursorTo(50, 3)).unwrap();
                    let to_print = format!("{}", "HMMM OUT:".on_green().black());
                    print!("{}", to_print);
                    w.act(Action::MoveCursorTo(50, 4)).unwrap();
                    let to_print = format!("{:<10}", reg_x_data);
                    print!("{}", to_print);
                } else {
                    println!("{}\n{}", "HMMM OUT:".on_green().black(), reg_x_data);
                }
                return Ok(());
            }
            "setn" => {
                let ending_data_i8 = signed_binary_conversion(
                    instruction_to_run.binary_contents[2..].join("").as_str(),
                );

                if ending_data_i8.is_err() {
                    return Err(RuntimeErr::InvalidData);
                }

                return self.write_rg(reg_x, ending_data_i8.unwrap() as i16);
            }
            "loadr" => {
                let data = self.read_mem(reg_y);

                if data.is_err() {
                    return Err(data.unwrap_err());
                }

                return self.write_rg(reg_x, data.unwrap());
            }
            "storer" => {
                return self.write_mem(reg_y, reg_x_data);
            }
            "popr" => {
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
            }
            "pushr" => {
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
                    return Err(mem_write.unwrap_err());
                }

                return self.write_rg(reg_y, reg_y_data + 1);
            }
            "loadn" => {
                let memory_data = self.read_mem(ending_data_u8);

                if memory_data.is_err() {
                    return Err(memory_data.unwrap_err());
                }
                let memory_data = memory_data.unwrap();
                return self.write_rg(reg_x, memory_data);
            }
            "storen" => {
                return self.write_mem(ending_data_u8, reg_x_data);
            }
            "addn" => {
                let ending_data_i8 = signed_binary_conversion(
                    instruction_to_run.binary_contents[2..].join("").as_str(),
                );

                if ending_data_i8.is_err() {
                    return Err(RuntimeErr::InvalidData);
                }

                return self.write_rg(reg_x, reg_x_data + ending_data_i8.unwrap() as i16);
            }
            "copy" => {
                let reg_y_data = self.read_rg(reg_y);

                if reg_y_data.is_err() {
                    return Err(reg_y_data.unwrap_err());
                }

                let reg_y_data = reg_y_data.unwrap();

                return self.write_rg(reg_x, reg_y_data);
            }
            "neg" => {
                let reg_y_data = self.read_rg(reg_y);

                if reg_y_data.is_err() {
                    return Err(reg_y_data.unwrap_err());
                }
                let reg_y_data = reg_y_data.unwrap();
                return self.write_rg(reg_x, -reg_y_data);
            }
            "add" | "sub" | "mul" | "div" | "mod" => {
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

                if reg_z_data == 0 && instruction_name == "div" {
                    return Err(RuntimeErr::DivideByZero);
                }

                let result: i16 = match instruction_name {
                    "add" => reg_y_data + reg_z_data,
                    "sub" => reg_y_data - reg_z_data,
                    "mul" => reg_y_data * reg_z_data,
                    "div" => reg_y_data / reg_z_data,
                    "mod" => reg_y_data % reg_z_data,
                    _ => 0,
                };

                return self.write_rg(reg_x, result);
            }
            "jumpr" => {
                if reg_x_data < 0 {
                    return Err(RuntimeErr::InvalidProgramCounter);
                }
                self.just_updated_pc = true;
                return self.update_pc(reg_x_data as usize);
            }
            "jumpn" => {
                self.just_updated_pc = true;
                return self.update_pc(ending_data_u8 as usize);
            }
            "jeqzn" => {
                if reg_x_data == 0 {
                    self.just_updated_pc = true;
                    return self.update_pc(ending_data_u8 as usize);
                } else {
                    Ok(())
                }
            }
            "jnezn" => {
                if reg_x_data != 0 {
                    self.just_updated_pc = true;
                    return self.update_pc(ending_data_u8 as usize);
                } else {
                    Ok(())
                }
            }
            "jgtzn" => {
                if reg_x_data > 0 {
                    self.just_updated_pc = true;
                    return self.update_pc(ending_data_u8 as usize);
                } else {
                    Ok(())
                }
            }
            "jltzn" => {
                if reg_x_data < 0 {
                    self.just_updated_pc = true;
                    return self.update_pc(ending_data_u8 as usize);
                } else {
                    Ok(())
                }
            }
            "calln" => {
                let update_rg = self.write_rg(reg_x, (pc + 1) as i16);

                if update_rg.is_err() {
                    return Err(update_rg.unwrap_err());
                }
                self.just_updated_pc = true;

                return self.update_pc(ending_data_u8 as usize);
            }
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
    pub fn get_counter_log(&self) -> Vec<usize> {
        return self.counter_log.clone();
    }
}

fn signed_binary_conversion(binary: &str) -> Result<i8, RuntimeErr> {
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
