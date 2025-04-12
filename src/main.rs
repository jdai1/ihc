mod solver;
mod lp;

use std::{env, io::BufRead, io::BufReader};
use std::fs::File;
use crate::solver::Solver;
use std::time::Instant;
use serde_json::json;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("usage: ipsolver <filename>");
        return;
    }

    let filename = args[1].clone();
    let mut solver = Solver::new(&filename);
    let start = Instant::now();
    let cost = solver.solve();
    let duration = start.elapsed();

    if cost == std::f64::MAX {
        let sol_dict = json!({
            "Instance": filename,
            "Time": duration.as_secs_f64(),
            "Result": "--",
            "Solution": "--"
        });
    
        println!("{}", sol_dict.to_string());
    } else {
        let sol_dict = json!({
            "Instance": filename,
            "Time": duration.as_secs_f64(),
            "Result": cost,
            "Solution": "OPT"
        });
    
        println!("{}", sol_dict.to_string());
    }
}
