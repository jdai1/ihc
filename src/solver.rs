use crate::lp::cplex::CPLEXSubprocessHandler;
use anyhow::{Result, anyhow};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

enum CPlexHandlerType {
    PYO3,
    SUBPROCESS
}

pub struct Solver {
    stack: Vec<i32>,
    trail: Vec<i32>,
    assignments: Vec<i32>, // len(num_tests); 1 (present), 0 (absent), -1 (unassigned)
    cplex_wrapper: CPLEXSubprocessHandler,
    incumbent_cost: f64,
    incumbent_assignment: Vec<f64>,
    ip_instance: IPInstance,
}

struct IPInstance {
    num_tests: usize,
    num_diseases: usize,
    cost_of_test: Vec<f64>,
    A: Vec<Vec<i32>>,
}

fn parse_data_file(filename: &String) -> Result<IPInstance, anyhow::Error> {
    let file = File::open(filename)?;
    let mut lines = BufReader::new(file).lines();

    let num_tests: usize = lines.next().unwrap().unwrap().trim().parse()?;
    let num_diseases: usize = lines.next().unwrap().unwrap().trim().parse()?;
    let cost_of_test: Vec<f64> = lines
        .next()
        .unwrap()
        .unwrap()
        .split_whitespace()
        .map(|s| s.parse::<f64>().expect("Failed to parse float"))
        .collect();

    let matrix: Vec<Vec<i32>> = (0..num_tests)
        .map(|_| {
            lines
                .next()
                .unwrap()
                .unwrap()
                .split_whitespace()
                .map(|s| s.parse::<i32>().expect("Failed to parse row in matrix"))
                .collect()
        })
        .collect();

    Ok(IPInstance {
        num_tests: num_tests,
        num_diseases: num_diseases,
        cost_of_test: cost_of_test,
        A: matrix,
    })
}

impl Solver {
    pub fn new(filename: &String) -> Self {
        let ip_instance = parse_data_file(filename).unwrap();
        Self {
            stack: Vec::new(),
            trail: Vec::new(),
            assignments: vec![-1; ip_instance.num_tests],
            cplex_wrapper: CPLEXSubprocessHandler::new(&filename),
            incumbent_cost: std::f64::MAX,
            incumbent_assignment: vec![0.0; ip_instance.num_tests],
            ip_instance: ip_instance
        }
    }

    pub fn solve(&mut self) -> f64 {
        let branch_var = self.get_branch_var();
        self.stack.push(-branch_var);
        self.stack.push(branch_var);
        while !self.stack.is_empty() {
            let next_var = self.stack.pop().unwrap();
            self.assign(next_var);
            let lp_sol = self.cplex_wrapper.solve_with_subprocess(&mut self.assignments);

            if !lp_sol.feasible {
                println!("Backtracking b/c infeasible");
                self.backtrack();
            } else if lp_sol.objective_value > self.incumbent_cost {
                println!("Backtracking b/c prunable branch");
                self.backtrack();
            } else if lp_sol.only_integral_assignments() {
                println!("Backtracking b/c all integral assignments");
                if lp_sol.objective_value < self.incumbent_cost {
                    println!("New incumbent: {}", lp_sol.objective_value);
                    self.incumbent_cost = lp_sol.objective_value;
                    self.incumbent_assignment = lp_sol.assignments;
                }
                self.backtrack();
            } else {
                let branch_var = self.get_branch_var();
                if branch_var == 0 {
                    // if all variables have been assigned, then backtrack
                    println!("Backtracking b/c all vars have been assigned");
                    self.backtrack();
                } else {
                    self.stack.push(-branch_var);
                    self.stack.push(branch_var);
                }
            }
        }

        self.incumbent_cost
    }

    fn assign(&mut self, var: i32) {
        self.assignments[(var.abs() - 1) as usize] = if var > 0 { 1 } else { 0 };
        self.trail.push(var);
    }

    fn get_branch_var(&mut self) -> i32 {
        for i in 0..self.assignments.len() {
            if self.assignments[i] == -1 {
                // println!("Branch var: {}", i+1);
                return (i + 1) as i32;
            }
        }
        return 0
    }

    fn backtrack(&mut self) {
        if self.stack.is_empty() {
            // if stack is empty, that means the entire space has been searched
            return;
        }
        let &next_var = self.stack.last().unwrap();
        loop {
            let var = self.trail.pop().unwrap().abs();
            self.assignments[(var - 1) as usize] = -1; // unassign variable
            if var == next_var.abs() {
                break;
            }
        }
    }
}
