use hmmm_rs;
use hmmm_rs::{write_file, load_file, compile_hmmm, write_compiled_hmmm, read_compiled_hmmm};
use hmmm_rs::simulator::*;

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