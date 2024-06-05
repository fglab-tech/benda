use parser::Parser;
use pyo3::prelude::*;
use pyo3::types::{PyFunction, PyString};
use rustpython_parser::{parse, Mode};
use types::u24::u24;
mod benda_ffi;
mod parser;
mod types;

#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

#[pyfunction]
fn switch() -> PyResult<String> {
    Ok("Ok".to_string())
}

#[pyfunction]
fn bjit(fun: Bound<PyFunction>, py: Python) -> PyResult<PyObject> {
    let (name, filename) = match fun.downcast::<PyFunction>() {
        Ok(inner) => {
            let name = inner.getattr("__name__");
            let filename =
                inner.getattr("__code__").unwrap().getattr("co_filename");

            (name.unwrap(), filename.unwrap())
        }
        Err(_) => todo!(),
    };

    let code = std::fs::read_to_string(filename.to_string()).unwrap();
    let module = parse(code.as_str(), Mode::Module, "main.py").unwrap();

    let mut val: Option<Bound<PyString>> = None;

    match module {
        rustpython_parser::ast::Mod::Module(mods) => {
            for stmt in mods.body.iter() {
                if let rustpython_parser::ast::Stmt::FunctionDef(fun_def) = stmt
                {
                    if fun_def.name == name.to_string() {
                        //let mut parser = Parser::new(mods.body.clone(), 0);

                        let mut parser = Parser::new(mods.body.clone(), 0);
                        let return_val = parser.parse(fun_def.name.as_ref());
                        val =
                            Some(PyString::new_bound(py, return_val.as_str()));
                    }
                }
            }
        }
        _ => unimplemented!(),
    }

    Ok(val.unwrap().to_object(py))
}

/// A Python module implemented in Rust.
#[pymodule]
fn benda(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_function(wrap_pyfunction!(bjit, m)?)?;
    m.add_function(wrap_pyfunction!(switch, m)?)?;
    m.add_class::<u24>()?;
    Ok(())
}
