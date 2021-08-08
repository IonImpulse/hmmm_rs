use super::simulator::*;
use super::*;
use std::fs;

// Maximum number of iterations the autograder will
// tolerate on each grade case before declaring the
// test failed.
const AUTOGRADER_MAX_ITERATIONS: u64 = 100000;

#[derive(Clone)]
pub struct TestCase {
    pub inputs: Vec<i16>,
    pub outputs: Vec<i16>,
}

impl TestCase {
    pub fn as_string(&self) -> String {
        let mut s = String::new();
        for i in &self.inputs {
            s = format!("{}{},", s, i);
        }

        s = s.trim_end_matches(",").to_string();
        s = format!("{}|", s);
        for i in &self.outputs {
            s = format!("{}{},", s, i);
        }
        s = s.trim_end_matches(",").to_string();

        s = format!("{};", s);

        s
    }
}

#[derive(Clone)]
pub struct GradeCase {
    sim: Option<Simulator>,
    test_case: Option<TestCase>,
    outputs: Vec<i16>,
    exit_code: i32,
    exit_name: String,
}

impl GradeCase {
    pub fn set_test_case(&mut self, test_case: TestCase) {
        self.test_case = Some(test_case);
    }

    pub fn get_test_case(&self) -> Option<TestCase> {
        self.test_case.clone()
    }

    pub fn test_case_matches(&self) -> bool {
        self.test_case.clone().unwrap().outputs == self.outputs
    }
}

#[derive(Clone)]
pub struct AutoGrader {
    pub file_names: Vec<String>,
    pub test_cases: Vec<TestCase>,
    pub grade_cases: Vec<GradeCase>,
    pub results: Vec<Vec<GradeCase>>,
}

impl AutoGrader {
    pub fn new_from_cmd(input_dir: &str, test_case_string: &str) -> Self {
        let mut test_cases: Vec<TestCase> = Vec::new();
        let test_case_string = test_case_string.trim_end_matches(";");
        if test_case_string.contains(";") {
            for test_case in test_case_string.split(";") {
                let test_case_split: Vec<String> =
                    test_case.split("|").map(|x| x.to_string()).collect();
                test_cases.push(TestCase {
                    inputs: test_case_split
                        .first()
                        .unwrap()
                        .split(",")
                        .map(|x| x.trim().parse::<i16>().unwrap())
                        .collect(),
                    outputs: test_case_split
                        .last()
                        .unwrap()
                        .split(",")
                        .map(|x| x.trim().parse::<i16>().unwrap())
                        .collect(),
                });
            }
        } else {
            let test_case_split: Vec<String> =
                test_case_string.split("|").map(|x| x.to_string()).collect();
            test_cases.push(TestCase {
                inputs: test_case_split
                    .first()
                    .unwrap()
                    .split(",")
                    .map(|x| x.trim().parse::<i16>().unwrap())
                    .collect(),
                outputs: test_case_split
                    .last()
                    .unwrap()
                    .split(",")
                    .map(|x| x.trim().parse::<i16>().unwrap())
                    .collect(),
            });
        }

        // Open dir and perform load_file on each .hmmm file
        let mut grade_cases: Vec<GradeCase> = Vec::new();
        let mut file_names: Vec<String> = Vec::new();
        for file in fs::read_dir(input_dir).unwrap() {
            let file_path = file.unwrap().path();
            if file_path.to_str().unwrap().ends_with(UNCOMPILED) {
                let input_file = load_file(file_path.to_str().unwrap().clone()).unwrap();
                let instructions = Simulator::compile_hmmm(input_file, true);
                let grade_case: GradeCase;

                file_names.push(
                    file_path
                        .as_path()
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .into(),
                );

                if instructions.is_err() {
                    grade_case = GradeCase {
                        sim: None,
                        test_case: None,
                        outputs: Vec::new(),
                        exit_code: instructions.clone().unwrap_err().as_code(),
                        exit_name: format!("{:?}", instructions.clone().unwrap_err()),
                    };
                } else {
                    grade_case = GradeCase {
                        sim: Some(Simulator::new_headless(instructions.unwrap())),
                        test_case: None,
                        outputs: Vec::new(),
                        exit_code: -1,
                        exit_name: "".to_string(),
                    };
                }

                grade_cases.push(grade_case);
            }
        }

        AutoGrader {
            file_names: file_names,
            test_cases: test_cases,
            grade_cases: grade_cases,
            results: Vec::new(),
        }
    }

    pub fn grade_all(&mut self) {
        let mut results: Vec<Vec<GradeCase>> = Vec::new();
        for test_case in self.test_cases.clone() {
            let mut test_case_results: Vec<GradeCase> = Vec::new();

            // Don't modify self, so we can reuse grade_cases
            let grade_cases = self.grade_cases.clone();

            let mut i = 0;
            for mut grade_case in grade_cases {
                grade_case.set_test_case(test_case.clone());
                
                println!(
                    "{} {} {} {}",
                    "Grading".bold().green(),
                    self.file_names[i],
                    "on test case".bold(),
                    test_case.as_string()
                );

                test_case_results.push(AutoGrader::grade_single(grade_case));
                
                i += 1;
            }
            results.push(test_case_results);
        }

        self.results = results;
    }

    pub fn grade_single(grade_case: GradeCase) -> GradeCase {
        let iterations_left = AUTOGRADER_MAX_ITERATIONS;
        let sim = grade_case.sim.clone();
        let test_case = grade_case.get_test_case().clone().unwrap();
        // If the simulator failed on compile, just return it
        if sim.is_none() {
            return grade_case;
        } else {
            let mut sim = sim.unwrap();
            sim.set_inputs(test_case.inputs.clone());

            while iterations_left > 0 {
                let step_result = sim.step();
                if step_result.is_err() {
                    let outputs = sim.get_outputs().clone();

                    return GradeCase {
                        sim: Some(sim),
                        test_case: Some(test_case),
                        outputs: outputs,
                        exit_code: step_result.clone().unwrap_err().as_code(),
                        exit_name: format!("{:?}", step_result.clone().unwrap_err()),
                    };
                }
            }
            let outputs = sim.get_outputs().clone();

            return GradeCase {
                sim: Some(sim),
                test_case: Some(test_case),
                outputs: outputs,
                exit_code: RuntimeErr::MaximumIterationsReached.as_code(),
                exit_name: format!("{:?}", RuntimeErr::MaximumIterationsReached),
            };
        }
    }

    pub fn print_results(&self) {
        let top_line =
            "█▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀█▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀█▀▀▀▀▀▀▀▀▀▀▀▀▀▀█▀▀▀▀▀▀▀▀▀▀▀▀▀▀█▀▀▀▀▀▀▀▀▀▀▀█";
        let bottom_line =
            "█▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄█▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄█▄▄▄▄▄▄▄▄▄▄▄▄▄▄█▄▄▄▄▄▄▄▄▄▄▄▄▄▄█▄▄▄▄▄▄▄▄▄▄▄█";

        println!("\n{}", "▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀".yellow());
        println!(
            "{}{}{}",
            "████".yellow(),
            "       GRADING SUCCESSFUL       ".green().bold(),
            "████".yellow()
        );
        println!("{}", "▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄".yellow());
        println!("\n");
        println!("{}", top_line);
        println!(
            "█ {}                                  █ {} █ {} █ {} █ {} █",
            "Name of File".bold(),
            "# Error Cases".bold(),
            "# Fail Cases".bold(),
            "# Pass Cases".bold(),
            "Pass/Fail".bold()
        );
        println!(
            "{}", bottom_line
        );
        print!("\n");
        println!("{}", top_line);
        
        for i in 0..self.results[0].len() {
            let grade_cases_all = self
                .results
                .iter()
                .map(|result| result[i].clone())
                .collect::<Vec<GradeCase>>();

            // Cases that failed with a runtime error
            let cases_errored: Vec<&GradeCase> = grade_cases_all
                .iter()
                .filter(|x| x.exit_code != 0)
                .collect();
            // Cases that did not match expected test case
            let cases_failed: Vec<&GradeCase> = grade_cases_all
                .iter()
                .filter(|x| x.exit_code == 0 && !x.test_case_matches())
                .collect();
            // Cases that passed
            let cases_passed: Vec<&GradeCase> = grade_cases_all
                .iter()
                .filter(|x| x.exit_code == 0 && x.test_case_matches())
                .collect();

            let pass_fail_emoji: ColoredString;

            if cases_passed.len() == self.results.len() {
                pass_fail_emoji = "P".to_string().bold().green();
            } else {
                pass_fail_emoji = "F".to_string().bold().red();
            }
            let output_string = format!(
                "█ {:45} █ {:13} █ {:12} █ {:12} █ {}  {:6} █",
                self.file_names[i],
                cases_errored.len(),
                cases_failed.len(),
                cases_passed.len(),
                pass_fail_emoji,
                format!("{}/{}",
                    cases_passed.len(),
                    self.results.len(),
                ).bold(),
            );

            println!("{}", output_string);
        }
        println!("{}", bottom_line);
    }
}
