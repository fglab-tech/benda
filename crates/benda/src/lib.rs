use std::borrow::Borrow;

use parser::Parser;
use pyo3::{ conversion::FromPyObjectBound, prelude::*, types::{ PyFunction, PyTuple } };
use python_ast::{parse, CodeGen, Name};
use types::u24::u24;
mod types;
mod parser;
mod bend;

#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

#[pyfunction]
fn bjit(fun: Bound<PyFunction>) -> PyResult<PyObject> {
    let (name, filename) = match fun.downcast::<PyFunction>() {
        Ok(inner) => {
            let name = inner.getattr("__name__");
            let filename = inner.getattr("__code__").unwrap().getattr("co_filename");

            (name.unwrap(), filename.unwrap())
        }
        Err(_) => todo!(),
    };

    let code = std::fs::read_to_string(filename.to_string()).unwrap();
    let ast = parse(&code, "").unwrap();

    for stmt in &ast.raw.body {

        match &stmt.statement {
            python_ast::StatementType::FunctionDef(fun_def) => {
                if fun_def.name == name.to_string() {
                    Parser::parse(fun_def.body.clone());
                }
            },
            _ => {},
        }
        };


    todo!();
}

/// A Python module implemented in Rust.
#[pymodule]
fn benda(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_function(wrap_pyfunction!(bjit, m)?)?;
    m.add_class::<u24>()?;
    Ok(())
}
