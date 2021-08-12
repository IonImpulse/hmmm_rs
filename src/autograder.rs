use super::simulator::*;
use super::*;
use csv;
use chrono;
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
        let mut output = String::new();
        for i in &self.inputs {
            output = format!("{}{},", output, i);
        }

        output = output.trim_end_matches(',').to_string();
        output = format!("{}|", output);
        for i in &self.outputs {
            output = format!("{}{},", output, i);
        }
        output = output.trim_end_matches(',').to_string();

        output = format!("{};", output);

        output
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

    pub fn passes(&self) -> bool {
        self.exit_code == 0 && self.test_case_matches()
    }

    pub fn passes_as_string(&self) -> String {
        if self.passes() {
            String::from("Pass")
        } else {
            String::from("Fail")
        }
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
        let test_case_string = test_case_string.trim_end_matches(';');
        if test_case_string.contains(';') {
            for test_case in test_case_string.split(';') {
                let test_case_split: Vec<String> =
                    test_case.split('|').map(|x| x.to_string()).collect();
                test_cases.push(TestCase {
                    inputs: test_case_split
                        .first()
                        .unwrap()
                        .split(',')
                        .map(|x| x.trim().parse::<i16>().unwrap())
                        .collect(),
                    outputs: test_case_split
                        .last()
                        .unwrap()
                        .split(',')
                        .map(|x| x.trim().parse::<i16>().unwrap())
                        .collect(),
                });
            }
        } else {
            let test_case_split: Vec<String> =
                test_case_string.split('|').map(|x| x.to_string()).collect();
            test_cases.push(TestCase {
                inputs: test_case_split
                    .first()
                    .unwrap()
                    .split(',')
                    .map(|x| x.trim().parse::<i16>().unwrap())
                    .collect(),
                outputs: test_case_split
                    .last()
                    .unwrap()
                    .split(',')
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
            file_names,
            test_cases,
            grade_cases,
            results: Vec::new(),
        }
    }

    pub fn grade_all(&mut self) {
        let mut results: Vec<Vec<GradeCase>> = Vec::new();
        for test_case in self.test_cases.clone() {
            println!("{} [{}]", "Grading Testcase".bold().blue(), test_case.as_string().bold());

            let mut test_case_results: Vec<GradeCase> = Vec::new();

            // Don't modify self, so we can reuse grade_cases
            let grade_cases = self.grade_cases.clone();

            let mut i = 0;
            for mut grade_case in grade_cases {
                grade_case.set_test_case(test_case.clone());
                
                let grade_result = AutoGrader::grade_single(grade_case);

                let grade_result_string: String;

                if grade_result.exit_code != 0 {
                    grade_result_string = format!("{} [{}]", "FAILED".bold().red(), grade_result.exit_name);
                } else if grade_result.exit_code == 0 && !grade_result.test_case_matches() {
                    grade_result_string = format!("{} [{}]", "FAILED".bold().red(), "Outputs don't match");
                } else {
                    grade_result_string = format!("{} [{}]", "PASSED".bold().green(), grade_result.exit_name);
                }

                test_case_results.push(grade_result);

                println!(
                    "- {} {:45} {} {}",
                    "Graded".bold().green(),
                    self.file_names[i],
                    ":".bold(),
                    grade_result_string,
                );
                i += 1;
            }
            results.push(test_case_results);
        }

        self.results = results;
    }

    pub fn grade_single(grade_case: GradeCase) -> GradeCase {
        let mut iterations_left = AUTOGRADER_MAX_ITERATIONS;
        let sim = grade_case.sim.clone();
        let test_case = grade_case.get_test_case().unwrap();
        // If the simulator failed on compile, just return it
        if sim.is_none() {
            grade_case
        } else {
            let mut sim = sim.unwrap();
            sim.set_inputs(test_case.inputs.clone());

            while iterations_left > 0 {
                let step_result = sim.step();
                if step_result.is_err() {
                    let outputs = sim.get_outputs();

                    return GradeCase {
                        sim: Some(sim),
                        test_case: Some(test_case),
                        outputs,
                        exit_code: step_result.clone().unwrap_err().as_code(),
                        exit_name: format!("{:?}", step_result.unwrap_err()),
                    };
                }

                iterations_left -= 1;
            }
            let outputs = sim.get_outputs();

            return GradeCase {
                sim: Some(sim),
                test_case: Some(test_case),
                outputs,
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
        println!();
        println!("{}", top_line);
        println!(
            "█ {}                                  █ {} █ {} █ {} █ {} █",
            "Name of File".bold(),
            "# Error Cases".bold(),
            "# Fail Cases".bold(),
            "# Pass Cases".bold(),
            "Pass/Fail".bold()
        );
        println!("{}", bottom_line);
        println!();
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
                .filter(|x| x.passes())
                .collect();

            let pass_fail_emoji: ColoredString;

            if cases_passed.len() == self.results.len() {
                pass_fail_emoji = "P".to_string().bold().green();
            } else {
                pass_fail_emoji = "F".to_string().bold().red();
            }
            let mut output_string = format!(
                "█ {:45} █ {:13} █ {:12} █ {:12} █ {}  {:6} █",
                self.file_names[i],
                cases_errored.len(),
                cases_failed.len(),
                cases_passed.len(),
                pass_fail_emoji,
                format!("{}/{}", cases_passed.len(), self.results.len(),),
            );

            // Increase readability by making every other line a different color
            if i % 2 == 0 {
                // "bright black" is just grey
                output_string = format!("{}", output_string.on_bright_black());
            }

            output_string = format!("{}", output_string.bold());

            println!("{}", output_string);
        }
        println!("{}", bottom_line);
    }

    /// Exports the GradeResults to a CSV file
    pub fn export_results(&self, path: &str) -> csv::Result<String> {
        let current_time = chrono::offset::Local::now();
        let current_time_string = current_time.format("%Y-%m-%d_%H-%M-%S");

        let out_path = format!("{}/results_{}.csv", path, current_time_string);
        let mut wtr = csv::WriterBuilder::new()
        .from_path(&out_path)?;

        // Write the header
        wtr.write_record(&["File Name", "Test Case", "Exit Code", "Exit String", "Pass/Fail"])?;

        // Write the results
        for i in 0..self.results[0].len() {
            for j in 0..self.results.len() {
                let grade_case = &self.results[j][i];
                wtr.write_record(&[
                    &self.file_names[i],
                    &grade_case.test_case.clone().unwrap().as_string(),
                    &grade_case.exit_code.to_string(),
                    &grade_case.exit_name,
                    &grade_case.passes_as_string(),
                ])?;
            }
        }
        
        
        wtr.flush()?;

        Ok(out_path)
    }
}
