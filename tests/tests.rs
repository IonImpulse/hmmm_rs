use hmmm_rs;
use hmmm_rs::{write_file, load_file, compile_hmmm, write_compiled_hmmm, read_compiled_hmmm};

#[test]
fn compile_uncompile_test() {
    // Create example code
    let test_hmmm = "
    0   read r1     # read dividend from the user\n
    1   write r1    # echo the input\n
    2   read r2     # read divisor from the user\n
    3   jeqzn r2, 7 # jump to 7 if trying to divide by 0\n
    
    4   div r3, r1, r2 # divide user's parameters\n
    5   write r3    # print the result\n
    6   halt\n
    
    7   setn r3, 0  # 0 is the result for division by 0\n
    8   write r3    # print the result\n
    9   halt";

    write_file("test.hmmm", test_hmmm).unwrap();

    // Load file
    let initial_file = load_file("test.hmmm").unwrap();
    // Compile
    let initial_compiled_file = compile_hmmm(initial_file);
    // Get the binary contents
    let binary_1: Vec<Vec<String>> = initial_compiled_file.iter().map(|x| x.binary_contents.clone()).collect();
    // Write compiled file
    write_compiled_hmmm("test.hb", initial_compiled_file.clone()).unwrap();
    // Load compiled file
    let compiled_file = load_file("test.hb").unwrap();
    // Uncompile
    let uncompiled_file = read_compiled_hmmm(compiled_file);
    // Get the binary contents
    let binary_2: Vec<Vec<String>> = uncompiled_file.iter().map(|x| x.binary_contents.clone()).collect();

    assert_eq!(binary_1, binary_2);
}
