use std::fs::File;
use std::io::{BufRead, BufReader};
use std::*;

use collections::HashMap;
use lazy_static::lazy_static;

static COMPILED: &str = ".hmmm";
static UNCOMPILED: &str = ".hb";

lazy_static!{
    static ref LOOKUP_MAP: HashMap<&'static str, u8> = vec![
        ("0000 0000 0000 0000", "1111 1111 1111 1111", "halt"),
        ("0000 0000 0000 0001", "1111 0000 1111 1111", "read"),
        ("0000 0000 0000 0010", "1111 0000 1111 1111", "write"),
        ("0000 0000 0000 0011", "1111 0000 1111 1111", "jumpr"),
        ("0001 0000 0000 0000", "1111 0000 0000 0000", "setn"),
        ("0010 0000 0000 0000", "1111 0000 0000 0000", "loadn"),
        ("0011 0000 0000 0000", "1111 0000 0000 0000", "storen"),
        ("0100 0000 0000 0000", "1111 0000 0000 1111", "loadr"),
        ("0100 0000 0000 0001", "1111 0000 0000 1111", "storer"),
        ("0100 0000 0000 0010", "1111 0000 0000 1111", "popr"),
        ("0100 0000 0000 0011", "1111 0000 0000 1111", "pushr"),
        ("0101 0000 0000 0000", "1111 0000 0000 0000", "addn"),
        ("0110 0000 0000 0000", "1111 1111 1111 1111", "nop"),
        ("0110 0000 0000 0000", "1111 0000 0000 1111", "copy"),
        ("0110 0000 0000 0000", "1111 0000 0000 0000", "add"),
        ("0111 0000 0000 0000", "1111 0000 1111 0000", "neg"),
        ("0111 0000 0000 0000", "1111 0000 0000 0000", "sub"),
        ("1000 0000 0000 0000", "1111 0000 0000 0000", "mul"),
        ("1001 0000 0000 0000", "1111 0000 0000 0000", "div"),
        ("1010 0000 0000 0000", "1111 0000 0000 0000", "mod"),
        ("1011 0000 0000 0000", "1111 1111 0000 0000", "jumpn"),
        ("1011 0000 0000 0000", "1111 0000 0000 0000", "calln"),
        ("1100 0000 0000 0000", "1111 0000 0000 0000", "jeqzn"),
        ("1101 0000 0000 0000", "1111 0000 0000 0000", "jnezn"),
        ("1110 0000 0000 0000", "1111 0000 0000 0000", "jgtzn"),
        ("1111 0000 0000 0000", "1111 0000 0000 0000", "jltzn"),
        ("0000 0000 0000 0000", "0000 0000 0000 0000", "data"),
    ].into_iter().collect();
}

let d = {"r0":"0000", "r1":"0001", "r2":"0010", "r3":"0011",
"r4":"0100", "r5":"0101", "r6":"0110", "r7":"0111",
"r8":"1000", "r9":"1001", "r10":"1010", "r11":"1011",
"r12":"1100", "r13":"1101", "r14":"1110", "r15":"1111", 
"R0":"0000", "R1":"0001", "R2":"0010", "R3":"0011",
"R4":"0100", "R5":"0101", "R6":"0110", "R7":"0111",
"R8":"1000", "R9":"1001", "R10":"1010", "R11":"1011",
"R12":"1100", "R13":"1101", "R14":"1110", "R15":"1111"}
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        panic!("Please specify a file to compile/run!")
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
                .map(|line| {let temp: Vec<u8> = line.split(" ").map(|i| u8::from_str_radix(i, 2).unwrap()).collect(); (temp[0],temp[1],temp[2],temp[3])})
                .collect();

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

fn compile_hmmm(uncompiled_text: Vec<String>) -> Vec<(u8,u8,u8,u8)> {
    let line_counter = 0;
    let compiled_text: Vec<(u8,u8,u8,u8)> = Vec::new();

    for line in uncompiled_text {
        if !(line.trim().starts_with("#")) {

        }
    }

    compiled_text
}