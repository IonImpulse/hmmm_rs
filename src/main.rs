use std::fs::File;
use std::io::{BufRead, BufReader};
use std::*;

use collections::HashMap;
use lazy_static::lazy_static;

static COMPILED: &str = ".hmmm";
static UNCOMPILED: &str = ".hb";

lazy_static!{
    static ref REGISTER_LOOKUP: HashMap<&'static str, &'static str> = vec![
        ("r0", "0000"), 
        ("r1", "0001"),
        ("r2", "0010"), 
        ("r3", "0011"),
        ("r4", "0100"),
        ("r5", "0101"),
        ("r6", "0110"),
        ("r7", "0111"),
        ("r8", "1000"),
        ("r9", "1001"),
        ("r10", "1010"),
        ("r11", "1011"),
        ("r12", "1100"),
        ("r13", "1101"),
        ("r14", "1110"),
        ("r15", "1111"),
    ].into_iter().collect();

    static ref INSTRUCTION_LOOKUP: Vec<InstructionType> = vec![
        InstructionType::new(vec!["halt"], "0000 0000 0000 0000", "1111 1111 1111 1111", ""),
        InstructionType::new(vec!["read"], "0000 0000 0000 0001", "1111 0000 1111 1111", "r"),
        InstructionType::new(vec!["write"], "0000 0000 0000 0010", "1111 0000 1111 1111", "r"),
        InstructionType::new(vec!["jumpr"], "0000 0000 0000 0011", "1111 0000 1111 1111", "r"),
        InstructionType::new(vec!["setn"], "0001 0000 0000 0000", "1111 0000 0000 0000", "r"),
        InstructionType::new(vec!["loadn"], "0010 0000 0000 0000", "1111 0000 0000 0000", ""),
        InstructionType::new(vec!["storen"], "0011 0000 0000 0000", "1111 0000 0000 0000", ""),
        InstructionType::new(vec!["loadr"], "0100 0000 0000 0000", "1111 0000 0000 0000", ""),
        InstructionType::new(vec!["storer"], "0100 0000 0000 0001", "1111 0000 0000 0000", ""),
        InstructionType::new(vec!["popr"], "0100 0000 0000 0010", "1111 0000 0000 1111", ""),
        InstructionType::new(vec!["pushr"], "0100 0000 0000 0011", "1111 0000 0000 1111", ""),
        InstructionType::new(vec!["addn"], "0101 0000 0000 0000", "1111 0000 0000 0000", ""),
        InstructionType::new(vec!["nop"], "0110 0000 0000 0000", "1111 1111 1111 1111", ""),
        InstructionType::new(vec!["copy"], "0110 0000 0000 0000", "1111 0000 0000 1111", ""),
        InstructionType::new(vec!["add"], "0110 0000 0000 0000", "1111 0000 0000 0000", ""),
        InstructionType::new(vec!["neg"], "0111 0000 0000 0000", "1111 0000 1111 0000", ""),
        InstructionType::new(vec!["sub"], "0111 0000 0000 0000", "1111 0000 0000 0000", ""),
        InstructionType::new(vec!["mul"], "1000 0000 0000 0000", "1111 0000 0000 0000", ""),
        InstructionType::new(vec!["div"], "1001 0000 0000 0000", "1111 0000 0000 0000", ""),
        InstructionType::new(vec!["mod"], "1010 0000 0000 0000", "1111 0000 0000 0000", ""),
        InstructionType::new(vec!["jumpn"], "1011 0000 0000 0000", "1111 1111 0000 0000", ""),
        InstructionType::new(vec!["calln"], "1011 0000 0000 0000", "1111 0000 0000 0000", ""),
        InstructionType::new(vec!["jeqzn"], "1100 0000 0000 0000", "1111 0000 0000 0000", ""),
        InstructionType::new(vec!["jnezn"], "1101 0000 0000 0000", "1111 0000 0000 0000", ""),
        InstructionType::new(vec!["jgtzn"], "1110 0000 0000 0000", "1111 0000 0000 0000", ""),
        InstructionType::new(vec!["jltzn"], "1111 0000 0000 0000", "1111 0000 0000 0000", ""),
        InstructionType::new(vec!["data"], "0000 0000 0000 0000", "0000 0000 0000 0000", ""), 
    ].into_iter().collect();
}

/// Struct for all instructions, to make it easier to
/// consolidate ones with aliases and order all of the
/// matching and masking strings
#[derive(PartialEq, Eq)]
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
    pub fn new(names: Vec<&'static str>, match_string: &'static str, mask_string: &'static str, arguments: &'static str) -> InstructionType {
        
        InstructionType {
            names: names,
            match_string: match_string,
            mask_string: mask_string,
            arguments: arguments,
        }
    }
}

pub struct Instruction {
    instruction_type: InstructionType,
    text_contents: &'static str,
    binary_contents: (u8, u8, u8, u8),
}

impl Instruction {
    pub fn new_from_text(contents: &'static str) -> Option<Instruction> {
        // Split on both "," and " "
        let contents_list: Vec<&'static str> = contents.split(&[',', ' '][..]).collect();

        let instruction_type: Option<InstructionType>;

        for instruction in INSTRUCTION_LOOKUP.into_iter() {
            if instruction.names.contains(&contents_list[0]) {
                instruction_type = Some(instruction);
                break;
            }
        }

        // First, check to make sure the instruction type exists in the lookup table
        if instruction_type.is_none() {
            return None
        }

        let instruction_type = instruction_type.unwrap();

        let instruction_args: Vec<&'static str> = contents_list[1..].into_iter().map(|a| a.clone()).collect();

        // Second, check to see if the number of arguments match
        if instruction_args.len() != instruction_type.arguments.len() {
            return None
        }
        
        let text_contents = instruction_args.join(" ");

        // Third, check if instructions match the source instruction types
        for (index, arg) in instruction_args.iter().enumerate() {
            
        }

        Some(Instruction {
            instruction_type: instruction_type,
            text_contents: text_contents,
            binary_contents: binary_contents,
        })
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        panic!("Please specify a file to compile/run!".to_string()
    } else if args.len() > 1 {
        let file_path: &str = &args[1];
        let mut uncompiled_text: Vec<String> = Vec::new();
        let mut compiled_text: Vec<(u8, u8, u8, u8)> = Vec::new();

        if file_path.ends_with(COMPILED) {
            uncompiled_text = load_hmmm_file(file_path).unwrap();

            compiled_text = compile_hmmm(uncompiled_text);
        } else if file_path.ends_with(UNCOMPILED) {
            compiled_text = load_hmmm_file(file_path)
                .unwrap()
                .into_iter()
                .map(|line| {let temp: Vec<u8> = line.split(" ".to_string().map(|i| u8::from_str_radix(i, 2).unwrap()).collect(); (temp[0],temp[1],temp[2],temp[3])})
                .collect();

        } else {
            panic!("Unknown filetype!".to_string();
        }
        if args.len() == 3 {
            if args[2] == "-o" {
                panic!("Please specify an output file!".to_string();
            }
        } else if args.len() == 4 {
            if args[0] == "-o" {}
        }
    }
}

fn load_hmmm_file(path: &str) -> std::io::Result<Vec<String>> {
    let reader = BufReader::new(File::open(path).expect("Cannot open file.txt".to_string());
    let mut output_vec: Vec<String> = Vec::new();
    for line in reader.lines() {
        output_vec.push(line?);
    }

    Ok(output_vec)
}

fn compile_hmmm(uncompiled_text: Vec<String>) -> Vec<(u8,u8,u8,u8)> {
    let line_counter = 0;
    let compiled_text: Vec<(u8,u8,u8,u8)> = Vec::new();

    for line in uncompiled_text {
        if !(line.trim().starts_with("#".to_string()) {

        }
    }

    compiled_text
}