mod lpsolver;
mod ipsolver;

use std::{env, time::Instant};
use serde_json::json;

use crate::lpsolver::LPSolver;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("usage: ipsolver <filename>");
        return;
    }

    let start = Instant::now();
    let filename = args[1].clone();

    let mut solver = ipsolver::IPSolver::new(&filename);
    let res = solver.solve();
    let duration = start.elapsed();

    if res.0 == std::f64::MAX {
        let sol_dict = json!({
            "Instance": filename,
            "Time": (duration.as_secs_f64() * 100.0).round() / 100.0,
            "Result": "--",
            "Solution": "--"
        });
    
        println!("{}", sol_dict.to_string());
    } else {
        let sol_dict = json!({
            "Instance": filename,
            "Time": (duration.as_secs_f64() * 100.0).round() / 100.0,
            "Result": res.0.round(),
            "Solution": "OPT"
        });
    
        println!("{}", sol_dict.to_string());
    }
}
