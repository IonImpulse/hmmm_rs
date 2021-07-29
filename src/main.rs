use clap::{App, Arg};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::process::*;
use std::{thread, time};

use colored::*;
use std::*;
use terminal::*;

mod simulator;
use simulator::*;

static UNCOMPILED: &str = ".hmmm";
static COMPILED: &str = ".hb";

/// Function that takes the current state of the terminal
/// and prints out only the lines that are different from the last time
/// we printed.
///
/// This is so that we can avoid printing the entire screen every time.

/// Function to load any text file as a Vec of Strings
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
    println!(
        "{}",
        "▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀".yellow().dimmed()
    );
    println!(
        "{}{}{}",
        "████".yellow(),
        "    COMPILATION UNSUCCESSFUL    ".red().on_yellow(),
        "████".yellow()
    );
    println!(
        "{}",
        "▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄".yellow().dimmed()
    );
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
    let current_line = sim.program_counter;
    println!(
        "{}",
        "▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀".yellow().dimmed()
    );
    println!(
        "{}{}{}",
        "████".yellow(),
        "    SIMULATION  UNSUCCESSFUL    ".red().on_yellow(),
        "████".yellow()
    );
    println!(
        "{}",
        "▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄".yellow().dimmed()
    );
    println!("\nERROR EXECUTING ADDRESS {}: {:?}", current_line, error);
    println!(
        "MEMORY ADDRESS CONTENTS: {} {}\n",
        sim.memory[current_line].instruction_type.names[0], sim.memory[current_line].text_contents
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
    println!("▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄\n");
    println!("▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀");
    println!("████    PROGRAM COUNTER HISTORY     ████");

    for (index, pc) in sim.counter_log.iter().enumerate() {
        println!("█ INSTRUCTION #{:8} █ {:8}", index + 1, pc);
    }
    println!("Exiting...");
    exit(1);
}

fn print_debug_screen(sim: &mut Simulator) -> terminal::error::Result<()> {
    let mut debug_screen_lines: Vec<String> = Vec::new();

    debug_screen_lines.push(format!(
        "{}",
        "█▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀█\n"
    ));

    debug_screen_lines.push(format!(
        "{}",
        "█             REGISTER CONTENTS             █\n"
    ));

    for row in 0..4 {
        debug_screen_lines.push(format!(
            "█    R{: <2}   █    R{: <2}   █    R{: <2}   █    R{: <2}   █\n",
            row * 4,
            (row * 4) + 1,
            (row * 4) + 2,
            (row * 4) + 3
        ));

        debug_screen_lines.push(format!(
            "█ {:8} █ {:8} █ {:8} █ {:8} █\n",
            sim.registers[row * 4],
            sim.registers[(row * 4) + 1],
            sim.registers[(row * 4) + 2],
            sim.registers[(row * 4) + 3]
        ));
    }
    debug_screen_lines.push(format!("{}", 
        "█▄▄▄▄▄▄▄▄▄▄█▄▄▄▄▄▄▄▄▄▄█▄▄▄▄▄▄▄▄▄▄█▄▄▄▄▄▄▄▄▄▄█▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄\n"
    ));
    debug_screen_lines.push(format!("{}", 
        "█    █   0  █   1  █   2  █   3  █   4  █   5  █   6  █   7  █   8  █   9  █   A  █   B  █   C  █   D  █   E  █   F  █\n"
    ));
    let address_chars = vec![
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F',
    ];

    let current_pc = &sim.program_counter;

    for (i, address_rows) in address_chars.iter().enumerate() {
        let mut to_print = format!("█  {} █", address_rows);

        for (j, _address_columns) in address_chars.iter().enumerate() {
            let memory_index = (i * 16) + j;

            let current_instruction = sim.memory[memory_index].clone();

            let instruction_text;
            if current_pc == &memory_index {
                instruction_text = current_instruction.as_hex().on_green();
            } else {
                if current_instruction.instruction_type.names[0] == "data" {
                    instruction_text = current_instruction.as_hex().on_black();
                } else {
                    instruction_text = current_instruction.as_hex().on_purple();
                }
            }

            to_print = format!("{} {} █", to_print, instruction_text);
        }

        debug_screen_lines.push(format!("{}\n", to_print));
    }

    debug_screen_lines.push(format!("{}", 
        "▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀\n"
    ));

    let mut w = terminal::stdout();
    w.act(Action::MoveCursorTo(0, 0))?;

    // Print line by line to avoid having to strobe the screen
    for line in debug_screen_lines {
        print!("{}", line);
    }

    w.flush()?;

    Ok(())
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
        // Print out startup message
        println!(
            "{}{}",
            "██    ██  ████    ████ ".yellow(),
            " ████    ████  ████    ████"
        );
        println!(
            "{}{}",
            "██    ██  ██ ██  ██ ██ ".yellow(),
            " ██ ██  ██ ██  ██ ██  ██ ██"
        );
        println!(
            "{}{}",
            "████████  ██  ████  ██ ".yellow(),
            " ██  ████  ██  ██  ████  ██"
        );
        println!(
            "{}{}",
            "██    ██  ██   ██   ██ ".yellow(),
            " ██   ██   ██  ██   ██   ██"
        );
        println!(
            "{}{}",
            "██    ██  ██        ██ ".yellow(),
            " ██        ██  ██        ██"
        );
        println!(
            "{}",
            " HARVEY       MUDD       MINIATURE      MACHINE   "
                .black()
                .dimmed()
                .italic()
                .bold()
                .on_white()
        );

        println!("\n");

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
        println!("{}", "▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀".yellow());
        println!(
            "{}{}{}",
            "████".yellow(),
            "     COMPILATION SUCCESSFUL     ".green().bold(),
            "████".yellow()
        );
        println!("{}", "▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄".yellow());
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
        if !matches.is_present("no-run") {
            // Create it as new struct from compiled HMMM
            let mut simulator = Simulator::new(compiled_text);
            if matches.is_present("debug") {
                println!("{}", "ENTERING DEBUGGING MODE...".on_red());
                simulator.debug = true;
                thread::sleep(time::Duration::from_millis(1000));
                terminal.act(Action::ClearTerminal(Clear::All))?;
                terminal.act(Action::DisableBlinking)?;
                terminal.act(Action::HideCursor)?;
            }

            loop {
                if simulator.debug == true {
                    print_debug_screen(&mut simulator)?;
                    thread::sleep(time::Duration::from_millis(500));
                }
                // Attempt to run a step in the simulator
                let result = &simulator.step();
                // If it's an error, raise it
                if result.is_err() {
                    let result_err = result.as_ref().unwrap_err();
                    // If the error is Halt, exit quietly, as that is the
                    // program successfully finishing
                    if result_err == &RuntimeErr::Halt {
                        if simulator.debug == true {
                            terminal.act(Action::MoveCursorTo(0, 31))?;
                        }

                        println!(
                            "{}",
                            "Program has reached end, exiting...".black().on_green()
                        );

                        exit(0);
                    } else {
                        // If not, raise that error!
                        terminal.act(Action::ClearTerminal(Clear::All))?;
                        raise_runtime_error(&simulator, &result_err);
                    }
                }
            }
        }

        Ok(())
    }
}
