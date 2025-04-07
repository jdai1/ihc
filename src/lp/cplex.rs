use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyDict, PyInt};
use std::ffi::CString;
use std::fs::File;
use std::io::Read;

pub struct CPLEXHandler {
    module: Py<PyModule>,
}

type TestNo = usize;

pub struct LPSolveResult {
    objective_value: f64, 
    assignments: Vec<f64>,
}

impl LPSolveResult {
    pub fn new(obj: f64, assignments: Vec<f64>) -> Self {
        LPSolveResult { objective_value: obj, assignments }
    }

    pub fn only_integral_assignments(&self) -> bool {
        self.assignments.iter().all(|f| !(f.fract() == 0.0))
    }
}

impl CPLEXHandler {
    const SOLVER_PY_PATH: &str = "src/ipinstance.py";

    pub fn new() -> Self {
        // save the correct stuff to call functions
        println!("cwd is {:?}", std::env::current_dir());
        let mut f = File::open(Self::SOLVER_PY_PATH).unwrap();
        let mut code = String::new(); 
        let read_res = f.read_to_string(&mut code).unwrap();
        assert!(read_res == f.metadata().unwrap().len() as usize);

        println!("read from file");

        Python::with_gil(|py|{
            // import things
            let cplex = PyModule::import(py, "docplex").unwrap();

            // turn code into a py module
            let file_name = Self::SOLVER_PY_PATH.split('/').last().unwrap();
            let module_name = file_name.split('.').next().unwrap();
            let module = PyModule::from_code(
                py, 
                &CString::new(code).unwrap(), 
                &CString::new("ipinstance.py").unwrap(), 
                &CString::new(module_name).unwrap()
            ).unwrap().into();

            println!("initialized module");

            // initialize model

            Self { module }
        })
    }

    /// Initialize the LP solver's model with the basic constraints for the problem.
    fn init_model(&mut self) {

    }

    /// Send a 
    pub fn solve(fixed: Vec<(TestNo, bool)>) -> LPSolveResult {
        todo!()
    }
}