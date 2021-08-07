use hmmm_rs;
use hmmm_rs::{write_file, load_file, compile_hmmm, write_compiled_hmmm, read_compiled_hmmm};
use hmmm_rs::simulator::*;

pub fn create_dummy_simulator() -> Simulator {
    // Load file
    let initial_file = load_file("tests/test.hmmm").unwrap();
    // Compile
    let compile_result = compile_hmmm(initial_file).unwrap();
    // Create simulator object
    Simulator::new(compile_result)
}

#[test]
fn compile_uncompile_binary_test() {
    // Load file
    let initial_file = load_file("tests/test.hmmm").unwrap();
    // Compile
    let initial_compiled_file = compile_hmmm(initial_file).unwrap();
    // Get the binary contents
    let binary_1: Vec<Vec<String>> = initial_compiled_file.iter().map(|x| x.binary_contents.clone()).collect();
    // Write compiled file
    write_compiled_hmmm("tests/test.hb", initial_compiled_file.clone()).unwrap();
    // Load compiled file
    let compiled_file = load_file("tests/test.hb").unwrap();
    // Uncompile
    let uncompiled_file = read_compiled_hmmm(compiled_file);
    // Get the binary contents
    let binary_2: Vec<Vec<String>> = uncompiled_file.iter().map(|x| x.binary_contents.clone()).collect();

    assert_eq!(binary_1, binary_2);
}

#[test]
fn does_not_compile_test() {
    // Load file
    let initial_file = load_file("tests/does_not_compile.hmmm").unwrap();
    // Compile - should fail
    let compile_result = compile_hmmm(initial_file);

    assert_eq!(compile_result.unwrap_err(), CompileErr::InvalidLineNumber);

}

#[test]
fn read_write_memory() {
    // Create simulator object
    let mut sim = create_dummy_simulator();
    // Write to memory
    sim.write_mem(100 as u8, 88 as i16).unwrap();
    // Read from memory
    assert_eq!(sim.read_mem(100).unwrap(), 88 as i16);
}

#[test]
fn read_write_register() {
    // Create simulator object
    let mut sim = create_dummy_simulator();
    // Write to register
    sim.write_reg(5 as u8, 88 as i16).unwrap();
    // Read from register
    assert_eq!(sim.read_reg(5).unwrap(), 88 as i16);
}

#[test]
fn perform_memory_register_operation_test() {
    // Create simulator object
    let mut sim = create_dummy_simulator();
    // Write to memory & register
    sim.write_mem(100 as u8, 88 as i16).unwrap();
    sim.write_reg(5 as u8, 88 as i16).unwrap();
    // Read memory
    let mem_result = sim.read_mem(100).unwrap();
    let reg_result = sim.read_reg(5).unwrap();
    // Perform operation
    sim.write_mem(101, mem_result * reg_result).unwrap();
    // Read from memory
    assert_eq!(sim.read_mem(101).unwrap(), 88 * 88 as i16);
}

#[test]
fn perform_halt_test() {
    // Create simulator object
    let mut sim = create_dummy_simulator();
    // Perform halt
    assert_eq!(sim.perform_halt(), Err(RuntimeErr::Halt));
}
