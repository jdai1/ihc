use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyDict, PyInt};
use std::ffi::CString;
use lp::cplex::{CPLEXHandler, LPSolveResult};

pub mod lp;

fn main() {
    // test_python_interp();
    // let func = PyFunctionHandler::new();\

    // for i in 0..10 {
    //     let val = func.call_incerement();
    //     println!("{i} -- val is {val:?}");
    // }
    let handler = CPLEXHandler::new();
    // let a = func.call_incerement();
    // println!("first is {a:?}");
    // let a = func.call_incerement();
    // println!("second is {a:?}");
}

struct PyFunctionHandler {
    module: Py<PyModule>,
}

impl PyFunctionHandler {
    fn new() -> Self {
        Python::with_gil(|py|{
            let code = r#"
state = {"counter": 0}

def increment(val):
    state["counter"] += int(val)
    return state["counter"]
"#;

            let module = PyModule::from_code(
                py, 
                &CString::new(code).unwrap(), 
                &CString::new("stateful.py").unwrap(), 
                &CString::new("stateful").unwrap()
            ).unwrap().into();
            Self { module }
        })
    }

    fn call_incerement(&self) -> i32 {
        let arg1 = "10";
        let a = Python::with_gil(|py|{
            
            let res = self.module.call_method1(py, "increment", (arg1,)).unwrap().extract::<i32>(py).unwrap();
            // let attr = self.module.getattr(py, "state[\"counter\"]").unwrap().extract::<i32>(py).unwrap();
            // let res = self.module.setattr(py, attr_name, value);
            // assert!(res == attr);
            res
        });
        a
    }
}