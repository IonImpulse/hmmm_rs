use hmmm_rs;
use hmmm_rs::{write_file, load_file, compile_hmmm, write_compiled_hmmm, read_compiled_hmmm};

#[test]
fn compile_uncompile_test() {
    // Load file
    let initial_file = load_file("tests/test.hmmm").unwrap();
    // Compile
    let initial_compiled_file = compile_hmmm(initial_file);
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
