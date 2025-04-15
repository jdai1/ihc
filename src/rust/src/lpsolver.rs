use cplex_rs::{
    Constraint, ConstraintType, Environment, ObjectiveType, Problem, ProblemType, Solution, Variable, VariableId, VariableType
};
use cplex_rs::parameters::Threads;
use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::ipsolver::common::FixedStatus;

pub struct ProblemInstance {
    pub num_tests: usize,
    pub num_diseases: usize,
    cost: Vec<usize>,
    A: Vec<Vec<usize>>,
}

pub fn parse_data_file(filename: &str) -> Result<ProblemInstance, anyhow::Error> {
    let file = File::open(filename)?;
    let mut lines = BufReader::new(file).lines();

    let num_tests: usize = lines.next().unwrap().unwrap().trim().parse()?;
    let num_diseases: usize = lines.next().unwrap().unwrap().trim().parse()?;
    let cost_of_test: Vec<usize> = lines
        .next()
        .unwrap()
        .unwrap()
        .split_whitespace()
        .map(|s| s.parse::<usize>().expect("Failed to parse float"))
        .collect();

    let matrix: Vec<Vec<usize>> = (0..num_tests)
        .map(|_| {
            lines
                .next()
                .unwrap()
                .unwrap()
                .split_whitespace()
                .map(|s| s.parse::<usize>().expect("Failed to parse row in matrix"))
                .collect()
        })
        .collect();

    Ok(ProblemInstance {
        num_tests: num_tests,
        num_diseases: num_diseases,
        cost: cost_of_test,
        A: matrix,
    })
}

pub struct LPSolver {
    problem_instance: ProblemInstance,
    table: Vec<Vec<usize>>,
    lp_problem: Problem,
    var_ids: Vec<VariableId>
}

impl LPSolver {
    pub fn new(filename: &str) -> Self {
        let inst = parse_data_file(filename).unwrap();

        let mut col_diffs: Vec<Vec<usize>> = Vec::new();
        for i in 0..inst.num_diseases {
            for j in (i + 1)..inst.num_diseases {
                let col_i: Vec<usize> = inst.A.iter().map(|row| row[i]).collect();
                let col_j: Vec<usize> = inst.A.iter().map(|row| row[j]).collect();

                let diff_sq: Vec<usize> = col_i
                    .iter()
                    .zip(col_j.iter())
                    .map(|(a, b)| (a != b) as usize)
                    .collect();

                col_diffs.push(diff_sq);
            }
        }

        // Transpose so each row corresponds to a test and columns to (i,j) pairs
        let num_tests = inst.A.len();
        let table: Vec<Vec<usize>> = (0..num_tests)
            .map(|row| col_diffs.iter().map(|col| col[row]).collect())
            .collect();

        let mut env = Environment::new().unwrap();

        // Setting global thread count
        env.set_parameter(Threads(1)).unwrap();

        let mut problem = Problem::new(env, "p4").unwrap();

        let usage_vars: Vec<Variable> = (0..inst.num_tests)
            .map(|x| Variable::new(VariableType::Continuous, 1.0, 0.0, 1.0, format!("{}", x)))
            .collect();
        let var_ids = problem.add_variables(usage_vars).unwrap();

        for col_idx in 0..table[0].len() {
            let col: Vec<f64> = table.iter().map(|row| row[col_idx] as f64).collect();
            let var_and_coeffs: Vec<(VariableId, f64)> = var_ids.iter().cloned().zip(col).collect();
            problem.add_constraint(Constraint::new(ConstraintType::GreaterThanEq, 1.0, None, var_and_coeffs)).unwrap();
        }

        let objective_func: Vec<(VariableId, f64)> = (0..inst.num_tests)
            .map(|x| (var_ids[x], inst.cost[x] as f64))
            .collect();
        let problem = problem
            .set_objective(ObjectiveType::Minimize, objective_func)
            .unwrap();

        LPSolver {
            problem_instance: inst,
            table: table,
            lp_problem: problem,
            var_ids: var_ids,
        }
    }

    pub fn solve(&mut self, assignments: &Vec<FixedStatus>) -> cplex_rs::Result<Solution> {
        assignments.iter().enumerate().for_each(|(i, &val)| {
            if val == FixedStatus::Present {
                self.lp_problem.fix_variable(self.var_ids[i], 1.0).unwrap();
            } else if val == FixedStatus::Absent {
                self.lp_problem.fix_variable(self.var_ids[i], 0.0).unwrap();
            }
        });
        
        let solution = self.lp_problem.solve_as(ProblemType::Linear);
        assignments.iter().enumerate().for_each(|(i, &val)| {
            if val != FixedStatus::Unassigned {
                self.lp_problem.unfix_variable(self.var_ids[i], 0.0, 1.0).unwrap();
            }
        });

        // TODO: think about how to add and unadd variables in more efficient way

        // println!("{:?}", solution);
        solution
    }
}
