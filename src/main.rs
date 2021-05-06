use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::*;

use std::*;
use lazy_static::lazy_static;

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
            vec!["jumpr"],
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
            vec!["copy"],
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
            vec!["calln"],
            "1011 0000 0000 0000",
            "1111 0000 0000 0000",
            "ru"
        ),
        InstructionType::new(
            vec!["jeqzn"],
            "1100 0000 0000 0000",
            "1111 0000 0000 0000",
            "ru"
        ),
        InstructionType::new(
            vec!["jnezn"],
            "1101 0000 0000 0000",
            "1111 0000 0000 0000",
            "ru"
        ),
        InstructionType::new(
            vec!["jgtzn"],
            "1110 0000 0000 0000",
            "1111 0000 0000 0000",
            "ru"
        ),
        InstructionType::new(
            vec!["jltzn"],
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
        } else if instruction_args.len() < instruction_type.arguments.len() {
            return Err(CompileErr::TooFewArguments);
        } else if instruction_type.arguments.len() == 0 {
            // If it's a single command, just return it
            return Ok(Instruction {
                instruction_type: instruction_type.clone(),
                text_contents: String::from(instruction_type.clone().names[0]),
                binary_contents: instruction_type.clone().match_string.split(" ").map(|a| String::from(a)).collect(),
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

        let mut instruction_chars = instruction_type.arguments.chars();

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
        for (index, arg) in instruction_args.iter().enumerate() {
            let current_instruction_type = instruction_chars.next().unwrap();

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
            }
            
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

            let mut matcher: Vec<String> = instruction
                .match_string
                .split(" ")
                .map(|a| String::from(a))
                .collect();

            let mut mask: Vec<bool> = instruction
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
        let mut text_contents = String::from(instruction_type.names[0]);

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
                let combined_binary = format!("{}{}",binary_contents[slots_filled],binary_contents[slots_filled + 1]);
                instruction_args.push(format!(
                    "{}",
                    i8::from_str_radix(combined_binary.as_str(), 2).unwrap()
                ));
                slots_filled += 2;
            } else if arg_type == 'u' {
                let combined_binary = format!("{}{}",binary_contents[slots_filled],binary_contents[slots_filled + 1]);
                instruction_args.push(format!(
                    "{}",
                    u8::from_str_radix(combined_binary.as_str(), 2).unwrap()
                ));
                slots_filled += 2;
            } else if arg_type == 'n' {
                let combined_binary = format!("{}{}",binary_contents[slots_filled],binary_contents[slots_filled + 1]);
                instruction_args.push(format!(
                    "{}",
                    i32::from_str_radix(combined_binary.as_str(), 2).unwrap()
                ));
                slots_filled += 3;
            }
        }
        if instruction_args.len() == 1 {
            text_contents = format!("{} {}", text_contents, instruction_args[0]);
        } else {
            for i in 0..(instruction_args.len()) {
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


fn load_hmmm_file(path: &str) -> std::io::Result<Vec<String>> {
    let reader = BufReader::new(File::open(path).expect("Cannot open file.txt"));
    let mut output_vec: Vec<String> = Vec::new();
    for line in reader.lines() {
        output_vec.push(line?);
    }

    Ok(output_vec)
}

fn raise_compile_error(line_num: usize, error: CompileErr, raw_line: &String, line_parts: Vec<String>) {
    let args: String = line_parts[2..].join(" ");
    println!("==================================");
    println!("==== COMPILATION UNSUCCESSFUL ====");
    println!("==================================\n");
    println!("ERROR ON LINE {}: {:?}", line_num, error);
    println!("Raw: \"{}\"", raw_line);
    println!("===========================================");
    println!("||           Interpreted As: ");
    println!("|| Line | Command | Arguments ");
    println!("|| {:4} | {:7} | {:15}", line_parts[0], line_parts[1], args);
    println!("===========================================");
    println!("Exiting...");
    exit(1); 
}

fn compile_hmmm(uncompiled_text: Vec<String>) -> Vec<Instruction> {
    let mut line_counter = 0;
    let mut compiled_text: Vec<Instruction> = Vec::new();

    for (index, line) in uncompiled_text.iter().enumerate() {
        if !(line.trim().starts_with("#")) && line.len() > 2 {
            let mut line_parts: Vec<String> = line.split(&[',', ' '][..]).map(|a| String::from(a)).collect();
            
            let line_number = line_parts.get(0).unwrap().trim().parse::<i128>();
            
            let comment_part = line_parts.iter().position(|a| a.starts_with("#"));

            if comment_part.is_some() {
                line_parts.drain(comment_part.unwrap()..);
            }

            let line_parts: Vec<String> = String::from(line_parts.join(" ").trim()).split_whitespace().map(|a| String::from(a)).collect();

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

fn read_compiled_hmm(raw_binary: Vec<String>) -> Vec<Instruction> {
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

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        panic!("Please specify a file to compile/run!")
    } else if args.len() > 1 {
        let file_path: &str = &args[1];
        let mut uncompiled_text: Vec<String> = Vec::new();
        let mut compiled_text: Vec<Instruction> = Vec::new();

        if file_path.ends_with(UNCOMPILED) {
            uncompiled_text = load_hmmm_file(file_path).unwrap();

            compiled_text = compile_hmmm(uncompiled_text);
        } else if file_path.ends_with(COMPILED) {
            let raw_binary = load_hmmm_file(file_path).unwrap();

            compiled_text = read_compiled_hmm(raw_binary);
        } else {
            panic!("Unknown filetype!");
        }

        if args.len() == 3 {
            if args[2] == "-o" {
                panic!("Please specify an output file!");
            }
        } else if args.len() == 4 {
            if args[0] == "-o" {}
        }
        
        println!("==================================");
        println!("====  COMPILATION SUCCESSFUL  ====");
        println!("==================================");
        println!("Line | Command | Arguments");

        for (index, line) in compiled_text.iter().enumerate() {
            if index > 9 {
                println!(".......");
                let last = compiled_text.last().unwrap();
                println!("{:4} | {:7} | {:15} ==>    {}", compiled_text.len() - 1, last.instruction_type.names[0], last.text_contents, last.binary_contents.join(" "));
                break;
            }
            println!("{:4} | {:7} | {:15} ==>    {}", index, line.instruction_type.names[0], line.text_contents, line.binary_contents.join(" "));
            
        }



    }
}
