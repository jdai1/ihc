use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyDict, PyInt};

use std::ffi::CString;
use std::fs::File;
use std::io::Read;
use std::process::Command;

pub struct LPSolveResult {
    pub feasible: bool,
    pub objective_value: f64,
    pub assignments: Vec<f64>,
}

impl LPSolveResult {
    pub fn new(obj: f64, assignments: Vec<f64>) -> Self {
        LPSolveResult {
            feasible: false,
            objective_value: obj,
            assignments,
        }
    }

    pub fn only_integral_assignments(&self) -> bool {
        self.assignments.iter().all(|f| f.fract() == 0.0)
    }
}

// pub struct CPLEXPyO3Handler {
//     module: Py<PyModule>,
// }

// impl CPLEXPyO3Handler {
//     const SOLVER_PY_PATH: &str = "src/ipinstance.py";

//     pub fn new(filename: &String) -> Self {
//         // save the correct stuff to call functions
//         println!("cwd is {:?}", std::env::current_dir());

//         let mut f = File::open(Self::SOLVER_PY_PATH).unwrap();
//         let mut code = String::new();
//         let read_res = f.read_to_string(&mut code).unwrap();
//         assert!(read_res == f.metadata().unwrap().len() as usize);

//         println!("read from file");

//         Python::with_gil(|py| {
//             // import things
//             let cplex = PyModule::import(py, "docplex").unwrap();

//             // turn code into a py module
//             let file_name = Self::SOLVER_PY_PATH.split('/').last().unwrap();
//             let module_name = file_name.split('.').next().unwrap();
//             let module = PyModule::from_code(
//                 py,
//                 &CString::new(code).unwrap(),
//                 &CString::new("ipinstance.py").unwrap(),
//                 &CString::new(module_name).unwrap(),
//             )
//             .unwrap()
//             .into();

//             println!("initialized module");

//             // initialize model

//             Self {
//                 module: module,
//             }
//         })
//     }

//     /// Initialize the LP solver's model with the basic constraints for the problem.
//     fn init_model(&mut self) {}

//     /// Send a
//     pub fn solve(&mut self, fixed: &mut [i32]) -> LPSolveResult {
//         todo!()
//     }
// }

pub struct CPLEXSubprocessHandler {
    filename: String,
}

impl CPLEXSubprocessHandler {
    pub fn new(filename: &String) -> Self {
        Self {
            filename: filename.clone(),
        }
    }

    pub fn solve_with_subprocess(&mut self, fixed: &mut [i32]) -> LPSolveResult {
        let output = Command::new("python3")
            .arg("./src/lp/python/main.py")
            .arg(&self.filename)
            .arg(format!("{:?}", fixed))
            .output()
            .expect("Failed to execute script");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        // println!("stdout: {}", stdout);
        // println!("stderr: {}", stderr);

        let lines: Vec<&str> = stdout.lines().collect();

        let feasible = lines.get(0).unwrap().trim().parse::<bool>().unwrap();
        if !feasible {
            return LPSolveResult {
                feasible: feasible,
                objective_value: 0.0,
                assignments: Vec::new(),
            };
        }

        let objective_value = lines.get(1).unwrap().trim().parse::<f64>().unwrap();
        let assignments: Vec<f64> = lines
            .get(2)
            .unwrap()
            .trim()
            .split_whitespace()
            .map(|s| s.parse::<f64>().unwrap())
            .collect();

        LPSolveResult {
            feasible: feasible,
            objective_value: objective_value,
            assignments: assignments,
        }
    }
}

// CPlex Sample

// use pyo3::prelude::*;
// use pyo3::types::{IntoPyDict, PyDict, PyInt};
// use std::ffi::CString;
// // use lp::cplex::{CPLEXHandler, LPSolveResult};

// fn main() {
//     let func = PyFunctionHandler::new();
//     for i in 0..10 {
//         let val = func.call_incerement();
//         println!("{i} -- val is {val:?}");
//     }
//     // let handler = CPLEXHandler::new();
//     let a = func.call_incerement();
//     println!("first is {a:?}");
//     let a = func.call_incerement();
//     println!("second is {a:?}");
// }

// struct PyFunctionHandler {
//     module: Py<PyModule>,
// }

// impl PyFunctionHandler {
//     fn new() -> Self {
//         Python::with_gil(|py|{
//             let code = r#"
// state = {"counter": 0}

// def increment(val):
//     state["counter"] += int(val)
//     return state["counter"]
// "#;

//             let module = PyModule::from_code(
//                 py,
//                 &CString::new(code).unwrap(),
//                 &CString::new("stateful.py").unwrap(),
//                 &CString::new("stateful").unwrap()
//             ).unwrap().into();
//             Self { module }
//         })
//     }

//     fn call_increment(&self) -> i32 {
//         let arg1 = "10";
//         let a = Python::with_gil(|py|{

//             let res = self.module.call_method1(py, "increment", (arg1,)).unwrap().extract::<i32>(py).unwrap();
//             // let attr = self.module.getattr(py, "state[\"counter\"]").unwrap().extract::<i32>(py).unwrap();
//             // let res = self.module.setattr(py, attr_name, value);
//             // assert!(res == attr);
//             res
//         });
//         a
//     }
// }
