mod lpsolver;

use std::{env, time::Instant};
use crate::lpsolver::LPSolver;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("usage: ipsolver <filename>");
        return;
    }

    let start = Instant::now();
    let filename = args[1].clone();
    let mut solver = LPSolver::new(&filename);
    let mut assignments = vec![-1; 25];
    for i in 0..100 {
        let cost = solver.solve(&assignments);
    }
    let duration = start.elapsed();
    println!("{:?}", duration.as_secs_f64());
    // if cost == std::f64::MAX {
    //     let sol_dict = json!({
    //         "Instance": filename,
    //         "Time": duration.as_secs_f64(),
    //         "Result": "--",
    //         "Solution": "--"
    //     });
    
    //     println!("{}", sol_dict.to_string());
    // } else {
    //     let sol_dict = json!({
    //         "Instance": filename,
    //         "Time": duration.as_secs_f64(),
    //         "Result": cost,
    //         "Solution": "OPT"
    //     });
    
    //     println!("{}", sol_dict.to_string());
    // }
}
