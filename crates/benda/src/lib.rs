use num_traits::ToPrimitive;
use parser::Parser;
use pyo3::prelude::*;
use pyo3::types::{PyFunction, PyString, PyTuple};
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
fn bjit(fun: Bound<PyFunction>, py: Python) -> PyResult<Py<PyAny>> {
    let arg_names_temp: Bound<PyAny>;

    let (name, filename, arg_names, argcount) =
        match fun.clone().downcast::<PyFunction>() {
            Ok(inner) => {
                let name = inner.getattr("__name__").unwrap();
                let code = inner.getattr("__code__").unwrap();
                let filename = code.getattr("co_filename").unwrap();

                arg_names_temp = code.getattr("co_varnames").unwrap();
                let arg_names = arg_names_temp.downcast::<PyTuple>().unwrap();
                let argcount = code
                    .getattr("co_argcount")
                    .unwrap()
                    .to_string()
                    .parse::<u32>()
                    .unwrap();

                (name, filename, arg_names, argcount)
            }
            Err(_) => todo!(),
        };

    let code = std::fs::read_to_string(filename.to_string()).unwrap();
    let module = parse(code.as_str(), Mode::Module, "main.py").unwrap();

    let mut arg_list: Vec<String> = vec![];

    for (index, arg) in arg_names.iter().enumerate() {
        if index >= argcount.to_usize().unwrap() {
            break;
        }

        arg_list.push(arg.to_string());
    }

    let mut val: Option<Bound<PyString>> = None;

    match module {
        rustpython_parser::ast::Mod::Module(mods) => {
            for stmt in mods.body.iter() {
                if let rustpython_parser::ast::Stmt::FunctionDef(fun_def) = stmt
                {
                    if fun_def.name == name.to_string() {
                        let mut parser = Parser::new(mods.body.clone(), 0);
                        let return_val =
                            parser.parse(fun_def.name.as_ref(), &arg_list);
                        val =
                            Some(PyString::new_bound(py, return_val.as_str()));
                    }
                }
            }
        }
        _ => unimplemented!(),
    }

    let fun: Py<PyAny> = PyModule::from_code_bound(
        py,
        format!(
            "def test({}):
            return {}",
            "tree",
            val.clone().unwrap()
        )
        .as_str(),
        "",
        "",
    )?
    .getattr("test")?
    .into();

    Ok(fun)
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
