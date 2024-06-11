use bend::imp;
use num_traits::ToPrimitive;
use parser::Parser;
use pyo3::{
    prelude::*,
    types::{PyDict, PyFunction, PyString, PyTuple},
};
use rustpython_parser::{parse, Mode};
use types::tree::Tree;
use types::{
    extract_type,
    tree::{Leaf, Node},
    u24::u24,
};
mod benda_ffi;
mod parser;
mod types;

#[pyfunction]
fn switch() -> PyResult<String> {
    Ok("Ok".to_string())
}

#[pyclass(name = "bjit")]
pub struct PyBjit {
    wraps: Py<PyAny>,
}

#[pymethods]
impl PyBjit {
    #[new]
    fn __new__(wraps: Py<PyAny>) -> Self {
        PyBjit { wraps }
    }
    #[pyo3(signature = (*args, **_kwargs))]
    fn __call__(
        &self,
        py: Python<'_>,
        args: &Bound<'_, PyTuple>,
        _kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Py<PyAny>> {
        let arg_names_temp: Bound<PyAny>;

        let (name, filename, arg_names, argcount) =
            match self.wraps.downcast_bound::<PyFunction>(py) {
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

        let mut arg_list: Vec<String> = vec![];

        for (index, arg) in arg_names.iter().enumerate() {
            if index >= argcount.to_usize().unwrap() {
                break;
            }

            arg_list.push(arg.to_string());
        }

        let mut parsed_types: Vec<(String, imp::Expr)> = vec![];

        for (index, arg) in args.downcast::<PyTuple>().unwrap().iter().enumerate() {
            parsed_types.push((
                arg_list.get(index).unwrap().to_string(),
                extract_type(arg).unwrap(),
            ));
        }

        let code = std::fs::read_to_string(filename.to_string()).unwrap();
        let module = parse(code.as_str(), Mode::Module, "main.py").unwrap();

        let mut val: Option<Bound<PyString>> = None;

        match module {
            rustpython_parser::ast::Mod::Module(mods) => {
                for (index, stmt) in mods.body.iter().enumerate() {
                    if let rustpython_parser::ast::Stmt::FunctionDef(fun_def) = stmt {
                        if fun_def.name == name.to_string() {
                            let mut parser =
                                Parser::new(mods.body.clone(), index, parsed_types.clone());
                            let return_val = parser.parse(fun_def.name.as_ref(), &[]);
                            val = Some(PyString::new_bound(py, return_val.as_str()));
                            break;
                        }
                    }
                }
            }
            _ => unimplemented!(),
        }

        Ok(val.unwrap().into())
    }
}

#[pyfunction]
fn bjit_test(fun: Bound<PyFunction>, py: Python) -> PyResult<Py<PyAny>> {
    let arg_names_temp: Bound<PyAny>;

    let (name, filename, arg_names, argcount) = match fun.clone().downcast::<PyFunction>() {
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

    let val: Option<Bound<PyString>> = None;

    match module {
        rustpython_parser::ast::Mod::Module(mods) => {
            for stmt in mods.body.iter() {
                if let rustpython_parser::ast::Stmt::FunctionDef(fun_def) = stmt {
                    if fun_def.name == name.to_string() {
                        //let mut parser = Parser::new(mods.body.clone(), 0);
                        //let return_val = parser.parse(fun_def.name.as_ref(), &arg_list);
                        //val = Some(PyString::new_bound(py, return_val.as_str()));
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

#[pymodule]
fn benda(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(bjit_test, m)?)?;
    m.add_function(wrap_pyfunction!(switch, m)?)?;
    m.add_class::<PyBjit>()?;
    m.add_class::<u24>()?;
    m.add_class::<Tree>()?;
    m.add_class::<Node>()?;
    m.add_class::<Leaf>()?;
    Ok(())
}
