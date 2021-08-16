use clap::{App, Arg};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::process::*;
use std::{thread, time};

use colored::*;
use std::*;
use terminal::*;

pub mod simulator;
pub mod autograder;
use simulator::*;
use autograder::*;

// File extension for HMMM files
// "Compiled" is really just a 1-to-1 mapping of the
// original file to binary, but it's more compact and
// does not support comments
static UNCOMPILED: &str = ".hmmm";
static COMPILED: &str = ".hb";

/// Function to load any text file as a Vec of Strings
pub fn load_file(path: &str) -> std::io::Result<Vec<String>> {
    let reader = BufReader::new(File::open(path).expect("Cannot open file"));
    let output_vec: Vec<String> = reader
        .lines()
        .map(|line| line.unwrap().trim().to_string())
        .collect();

    Ok(output_vec)
}

/// Function to pretty-print a compilation error and exit
/// the program gracefully
pub fn raise_compile_error(
    line_num: usize,
    error: CompileErr,
    raw_line: &String,
    line_parts: Vec<String>,
) {
    let args: String = line_parts[2..].join(" ");

    println!("{}", "▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀".yellow());
    println!(
        "{}{}{}",
        "████".yellow(),
        "    COMPILATION UNSUCCESSFUL    ".red().bold(),
        "████".yellow()
    );
    println!("{}\n", "▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄".yellow());

    println!(
        "{} {:?}",
        format!("{} {}:", " ERROR ON LINE", line_num)
            .on_red()
            .white()
            .bold(),
        error,
    );

    println!(
        "{} \"{}\"\n",
        " RAW TEXT:".on_red().white().bold(),
        raw_line.white(),
    );
    println!("█▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀");
    println!("█           Interpreted As: ");
    println!("█ Line █ Command █ Arguments ");
    println!("█ {:4} █ {:7} █ {:15}", line_parts[0], line_parts[1], args);
    println!("█▄▄▄▄▄▄█▄▄▄▄▄▄▄▄▄█▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄");
    println!("Exiting...");
}

/// Function to pretty-print a runtime error and exit
/// the program gracefully
pub fn raise_runtime_error(sim: &Simulator, error: &RuntimeErr) {
    // Easy way to display information: show the debug screen!
    let _debug_result = print_debug_screen(sim);
    let current_line = sim.get_program_counter();

    let w = terminal::stdout();
    w.act(Action::MoveCursorTo(0, 29)).unwrap();
    println!("{}", "▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀".yellow());
    println!(
        "{}{}{}",
        "████".yellow(),
        "    SIMULATION UNSUCCESSFUL     ".red().bold(),
        "████".yellow()
    );
    println!("{}\n", "▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄".yellow());

    println!(
        "{} {:?}",
        format!("{} {}:", " ERROR EXECUTING ADDRESS", current_line)
            .on_red()
            .white()
            .bold(),
        error
    );
    let current_line_contents = sim.get_memory(current_line).unwrap();
    println!(
        "{} {} {}\n",
        " MEMORY ADDRESS CONTENTS:".on_red().white().bold(),
        current_line_contents.instruction_type.names[0],
        current_line_contents.text_contents
    );
}

/// Function to print the current state of the simulator
/// (registers, memory, etc.) to the screen without flickering
/// (i.e. no flicker when the screen is updated)
pub fn print_debug_screen(sim: &Simulator) -> terminal::error::Result<()> {
    let mut debug_screen_lines: Vec<String> = Vec::new();

    debug_screen_lines.push(format!(
        "{}{}{}",
        "█▀▀▀▀▀▀▀▀▀▀█",
        "  REGISTER CONTENTS  ".bold().on_blue(),
        "█▀▀▀▀▀▀▀▀▀▀█\n",
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
            &sim.get_register(row * 4).unwrap_or(0),
            &sim.get_register((row * 4) + 1).unwrap_or(0),
            &sim.get_register((row * 4) + 2).unwrap_or(0),
            &sim.get_register((row * 4) + 3).unwrap_or(0),
        ));
    }
    debug_screen_lines.push("█▄▄▄▄▄▄▄▄▄▄█▄▄▄▄▄▄▄▄▄▄█▄▄▄▄▄▄▄▄▄▄█▄▄▄▄▄▄▄▄▄▄█▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄\n".to_string());
    debug_screen_lines.push("█    █   0  █   1  █   2  █   3  █   4  █   5  █   6  █   7  █   8  █   9  █   A  █   B  █   C  █   D  █   E  █   F  █\n".to_string());
    let address_chars = vec![
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F',
    ];

    let current_pc = &sim.get_program_counter();

    for (i, address_rows) in address_chars.iter().enumerate() {
        let mut to_print = format!("█  {} █", address_rows);

        for (j, _address_columns) in address_chars.iter().enumerate() {
            let memory_index = (i * 16) + j;

            // We can safely unwrap here because we know that the memory
            // is fully populated. Any errors will be caught at compile time,
            // or worst case, with a RuntimeErr.
            let current_instruction = sim.get_memory(memory_index).unwrap();

            let instruction_text;
            if current_pc == &memory_index {
                instruction_text = current_instruction.as_hex().on_green();
            } else if current_instruction.instruction_type.names[0] == "data" {
                if current_instruction.binary_contents == vec!["0000", "0000", "0000", "0000"] {
                    instruction_text = current_instruction.as_hex().on_black();
                } else {
                    instruction_text = current_instruction.as_hex().on_yellow().black();
                }
            } else {
                instruction_text = current_instruction.as_hex().on_purple();
            }

            to_print = format!("{} {} █", to_print, instruction_text);
        }

        debug_screen_lines.push(format!("{}\n", to_print));
    }

    debug_screen_lines.push("▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀\n".to_string());

    // Create terminal object to print out the debug screen
    let mut w = terminal::stdout();
    // Make sure the cursor is at the top of the screen
    w.act(Action::MoveCursorTo(0, 0))?;
    // Print line by line to avoid having to strobe the screen
    for line in debug_screen_lines {
        print!("{}", line);
    }

    // Line by line printing done, now to print out program counter,
    // IR, human-readable output, and HMMM output.

    // Print program counter
    w.act(Action::MoveCursorTo(50, 1)).unwrap();
    let to_print = format!("{}", " PROGRAM COUNTER: ".on_red().white().bold());
    print!("{}", to_print);
    w.act(Action::MoveCursorTo(50, 2)).unwrap();
    let to_print = format!("{:<10}", sim.get_program_counter());
    print!("{}", to_print);

    // Print IR
    w.act(Action::MoveCursorTo(50, 4)).unwrap();
    let to_print = format!("{}", " INSTRUCTION REGISTER: ".on_red().white().bold());
    print!("{}", to_print);
    let memory_ir = sim.get_memory(sim.get_program_counter());

    if memory_ir.is_some() {
        let memory_ir = memory_ir.unwrap();
        w.act(Action::MoveCursorTo(50, 5)).unwrap();
        let to_print = format!(
            "{:<15}",
            format!(
                "{} {}",
                memory_ir.instruction_type.names[0], memory_ir.text_contents
            )
        );
        print!("{}", to_print);

        // Print human-readable output
        w.act(Action::MoveCursorTo(75, 1)).unwrap();
        let to_print = format!("{}", " HUMAN-READABLE CODE: ".on_green().white().bold());
        print!("{}", to_print);

        w.act(Action::MoveCursorTo(75, 2)).unwrap();

        let mut to_print = String::from(memory_ir.instruction_type.human_explanation);

        for (i, c) in memory_ir.instruction_type.arguments.chars().enumerate() {
            let result: String = match c {
                'r' => {
                    format!(
                        "{}",
                        u8::from_str_radix(memory_ir.binary_contents[i + 1].as_str(), 2).unwrap()
                    )
                }
                's' => {
                    let converted = signed_binary_conversion(
                        memory_ir.binary_contents[i + 1..i + 3].join("").as_str(),
                    )
                    .unwrap();

                    format!("{}", converted)
                }
                'u' => {
                    let converted = u8::from_str_radix(
                        memory_ir.binary_contents[i + 1..i + 3].join("").as_str(),
                        2,
                    )
                    .unwrap();
                    format!("{}", converted)
                }
                'n' => {
                    let converted =
                        i32::from_str_radix(memory_ir.binary_contents.join("").as_str(), 2)
                            .unwrap();
                    format!("{}", converted)
                }
                _ => String::from(""),
            };
            if !result.is_empty() {
                to_print = to_print.replacen("_", result.as_str(), 1);
            }
        }
        if to_print.len() > 45 {
            print!(
                "{:<45}",
                to_print.drain(..40).collect::<String>().trim().bold()
            );
            w.act(Action::MoveCursorTo(75, 3)).unwrap();
            print!("{:<45}", to_print.trim().bold());
        } else {
            print!("{:<45}", to_print.bold());
            w.act(Action::MoveCursorTo(75, 3)).unwrap();
            print!("{:<45}", "");
        }
    }

    // Print HMMM output
    w.act(Action::MoveCursorTo(50, 7)).unwrap();
    let to_print = format!("{}", " HMMM OUT: ".on_green().white().bold());
    print!("{}", to_print);

    w.flush()?;

    Ok(())
}

/// Function to read a vec of binary HMMM text into
/// a Vec of Instruction structs
pub fn read_compiled_hmmm(raw_binary: Vec<String>) -> Vec<Instruction> {
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
pub fn write_uncompiled_hmmm(path: &str, compiled_text: Vec<Instruction>) -> std::io::Result<()> {
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
pub fn write_compiled_hmmm(path: &str, compiled_text: Vec<Instruction>) -> std::io::Result<()> {
    let mut contents = String::from("");

    for instruction in compiled_text {
        let binary = instruction.binary_contents.join(" ");
        contents = format!("{}{}\n", contents, binary);
    }

    contents = String::from(contents.trim_end());

    fs::write(path, contents)?;
    Ok(())
}

pub fn write_file(path: &str, contents: &str) -> std::io::Result<()> {
    fs::write(path, contents)?;
    Ok(())
}

pub fn main() -> terminal::error::Result<()> {
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
        .arg(Arg::with_name("speed")
                 .short("s")
                 .long("speed")
                 .takes_value(true)
                 .help("Sets the multiplier (speed) of debug mode (eg: .5 is half speed, 2 is double)"))
        .arg(Arg::with_name("autograder")
                 .short("a")
                 .long("autograder")
                 .takes_value(true)
                 .help("Toggles the AutoGrader functionality, expecting a test string to be given. If enabled, expects a directory path instead of a file path for --input and --output. --debug, --no-run, and --speed are ignored in this mode."))         
        .get_matches();

    if matches.value_of("input").is_none() {
        println!("Error: Please specify a file to compile/run!");
        exit(1);
    } else {
        // Print out startup message
        println!(
            "{} ████    ████  ████    ████",
            "██    ██  ████    ████ ".yellow()
        );
        println!(
            "{} ██ ██  ██ ██  ██ ██  ██ ██",
            "██    ██  ██ ██  ██ ██ ".yellow()
        );
        println!(
            "{} ██  ████  ██  ██  ████  ██",
            "████████  ██  ████  ██ ".yellow()
        );
        println!(
            "{} ██   ██   ██  ██   ██   ██",
            "██    ██  ██   ██   ██ ".yellow()
        );
        println!(
            "{} ██        ██  ██        ██",
            "██    ██  ██        ██ ".yellow()
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

        println!();

        let file_path: &str = matches.value_of("input").unwrap().trim_start_matches(".\\");

        if matches.value_of("autograder").is_some() {
            println!("{}\n", "AutoGrader Mode Enabled".bold().on_green());
            let path = file_path.trim_matches(&['\\', '/'] as &[_]);
            let mut autograder = AutoGrader::new_from_cmd(path, matches.value_of("autograder").unwrap());
            autograder.grade_all();
            autograder.print_results();
            let export_result = autograder.export_results(path);

            if export_result.is_err() {
                println!("\n{}\n", "AutoGrader Export Failed".bold().on_red());
            } else {
                println!("\n{} {}\n", "AutoGrader Export Successful:".bold().on_green(), export_result.unwrap().bold());
            }
            exit(0);
        }

        // Setup the vec for the compiled Instructions
        let compiled_text: Vec<Instruction>;

        // Check to see what type of file is being loaded
        if file_path.ends_with(UNCOMPILED) {
            // If it's uncompiled, load it
            let uncompiled_text = load_file(file_path).unwrap();

            // Then, compile it into Instruction structs
            let compile_result = Simulator::compile_hmmm(uncompiled_text, false);

            if compile_result.is_err() {
                exit(compile_result.unwrap_err().as_code())
            } else {
                compiled_text = compile_result.unwrap();
            }
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
            let debug_multiplier = matches
                .value_of("speed")
                .unwrap_or("1")
                .parse::<f64>()
                .unwrap_or(1.0);

            if matches.is_present("debug") {
                println!("{}", "ENTERING DEBUGGING MODE...".on_red());
                simulator.set_debug(true);
                thread::sleep(time::Duration::from_millis(
                    (1000. / debug_multiplier) as u64,
                ));
                terminal.act(Action::ClearTerminal(Clear::All))?;
                terminal.act(Action::DisableBlinking)?;
                terminal.act(Action::HideCursor)?;
            }

            loop {
                if simulator.is_debug() {
                    print_debug_screen(&mut simulator)?;
                    thread::sleep(time::Duration::from_millis(
                        (500. / debug_multiplier) as u64,
                    ));
                }
                // Attempt to run a step in the simulator
                let result = &simulator.step();
                // If it's an error, raise it
                if result.is_err() {
                    // Don't trap the user without a cursor,
                    // make sure to show it on exit
                    // Hopefully the program doesn't hard crash because if it does,
                    // the cursor might not be visible
                    terminal.act(Action::ShowCursor)?;
                    let result_err = result.as_ref().unwrap_err();
                    // If the error is Halt, exit quietly, as that is the
                    // program successfully finishing
                    if result_err == &RuntimeErr::Halt {
                        if simulator.is_debug() {
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
                        // Prints out the debug screen as well as the the error
                        raise_runtime_error(&simulator, result_err);
                        let exit_code = &result_err.as_code();

                        // Move the terminal prompt to the bottom of the screen
                        for _ in 0..16 {
                            println!("\n");
                        }
                        exit(*exit_code);
                    }
                }
            }
        }
        Ok(())
    }
}
