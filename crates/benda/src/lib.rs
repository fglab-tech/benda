//! Benda Module
//!
//! This module provides functionality for working with Bend code, including loading and parsing
//! Bend books, and compiling Python functions to Bend code at runtime.
//!
//! The module includes functions for loading Bend books from strings or files, as well as
//! the `PyBjit` class, which serves as an annotation for compiling Python functions to Bend code.
//!
//! # Key Components
//!
//! - `load_book`: Load a Bend book from a string of Bend code
//! - `load_book_from_file`: Load a Bend book from a file
//! - `PyBjit`: Annotation class for compiling Python functions to Bend code at runtime
//!
//! # Examples
//!
//! ```python
//! from benda import bjit, load_book
//!
//! # Load a Bend book from a string
//! book = load_book("some bend code here")
//!
//! # Use the bjit annotation to compile a Python function to Bend code
//! @bjit
//! def add(x, y):
//!     return x + y
//!
//! result = add(1, 2)  # This will execute the Bend-compiled version of the function
//! print(result)  # Output: 3
//!
//! # Load a Bend book from a file
//! book_from_file = load_book_from_file("/path/to/bend/file.bend")
//! ```
//!
//! # Note
//!
//! This module integrates Python and Bend, allowing for seamless interaction between the two languages.
//! It's particularly useful for projects that need to leverage Bend's capabilities while working within a Python environment.

use std::path::Path;

use num_traits::ToPrimitive;
use parser::Parser;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyFunction, PyString, PyTuple};
use rustpython_parser::{parse, Mode};
use types::book::{BendCommand, Book};
use types::fan::Fan;
use types::tree::{Leaf, Node, Tree};
use types::u24::U24;
mod benda_ffi;
mod parser;
mod types;

#[pyfunction]
fn switch() -> PyResult<String> {
    Ok("Ok".to_string())
}

/// Load a Book from a string of Bend code
///
/// This function parses a string of Bend code and creates a `Book` instance.
///
/// # Arguments
///
/// * `code` - A Python String containing the Bend code
///
/// # Returns
///
/// Returns a `Book` instance.
///
/// # Examples
///
/// ```python
/// book = benda.load_book("def Example: ...")
/// book.defs.Example()
/// ```
#[pyfunction]
fn load_book(py: Python, code: Py<PyString>) -> PyResult<Py<Book>> {
    let builtins = bend::fun::Book::builtins();
    let path = Path::new("./tmp/bend_book.tmp");
    let bend_book = bend::fun::load_book::do_parse_book(
        code.to_string().as_str(),
        path,
        builtins,
    );

    let book = Book::new(&mut bend_book.unwrap());

    Ok(Py::new(py, book).unwrap())
}

/// Load a Book from a file
///
/// This function reads a Bend code file and creates a `Book` instance.
///
/// # Arguments
///
/// * `path` - A Python String containing the path to the Bend code file
///
/// # Returns
///
/// Returns a `Book` instance.
///
/// # Examples
///
/// ```python
/// book = benda.load_book_from_file("./path/to/file.bend")
/// book.defs.Example()
/// ```
#[pyfunction]
fn load_book_from_file(py: Python, path: Py<PyString>) -> PyResult<Py<Book>> {
    let binding = path.to_string();
    let new_path = Path::new(&binding);
    let bend_book = bend::load_file_to_book(new_path);

    let book = Book::new(&mut bend_book.unwrap());

    Ok(Py::new(py, book).unwrap())
}

/// Bjit decorator
///
/// A Python class that serves as an decorator to compile Python functions to Bend code at runtime.
///
/// # Fields
///
/// * `wraps` - A `function` representing the wrapped Python function
///
/// # Examples
///
/// ```python
/// from benda import bjit
///
/// @bjit
/// def my_function(x, y):
///     return x + y
///
/// result = my_function(1, 2)  # This will execute the Bend-compiled version of the function
/// ```
#[pyclass(name = "bjit")]
pub struct PyBjit {
    wraps: Py<PyAny>,
}

#[pymethods]
impl PyBjit {
    /// Create a new PyBjit instance
    ///
    /// # Arguments
    ///
    /// * `wraps` - A Python function to be compiled to Bend code
    ///
    /// # Returns
    ///
    /// Returns a new `bjit` instance that wraps the given Python function.
    #[new]
    fn __new__(wraps: Py<PyAny>) -> Self {
        PyBjit { wraps }
    }
    /// Call the Bend-compiled version of the wrapped Python function
    ///
    /// This method implements the runtime compilation and execution of the wrapped Python function.
    /// It analyzes the function's code, compiles it to Bend code, and then executes the compiled version.
    ///
    /// # Arguments
    ///
    /// * `args` - A tuple of positional arguments to be passed to the function
    ///
    /// # Returns
    ///
    /// Returns a `Any` containing the result of the Bend-compiled function execution.
    ///
    /// # Process
    ///
    /// 1. Extracts function metadata (name, filename, argument names, etc.)
    /// 2. Parses the function's Python code
    /// 3. Compiles the Python code to Bend code
    /// 4. Executes the compiled Bend code with the provided arguments
    /// 5. Returns the result of the execution
    ///
    /// # Notes
    ///
    /// - The compilation process occurs at runtime, which may introduce some overhead on the first call.
    /// - Subsequent calls to the same function may benefit from caching, though this is not explicitly implemented in the current version.
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
                    let arg_names =
                        arg_names_temp.downcast::<PyTuple>().unwrap();
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

        let mut parsed_types: Vec<(String, Bound<PyAny>)> = vec![];

        for (index, arg) in
            args.downcast::<PyTuple>().unwrap().iter().enumerate()
        {
            let var_name = arg_list.get(index).unwrap().to_string();

            parsed_types.push((var_name.clone(), arg));
        }

        let code = std::fs::read_to_string(filename.to_string()).unwrap();
        let module = parse(code.as_str(), Mode::Module, "main.py").unwrap();

        let mut val: Option<Bound<PyString>> = None;

        match module {
            rustpython_parser::ast::Mod::Module(mods) => {
                for stmt in mods.body.iter() {
                    if let rustpython_parser::ast::Stmt::FunctionDef(fun_def) =
                        stmt
                    {
                        if fun_def.name == name.to_string() {
                            let mut parser = Parser::new(
                                mods.body.clone(),
                                parsed_types.clone(),
                            );
                            let return_val =
                                parser.parse(name.to_string().as_ref(), &[]);

                            match return_val {
                                Ok(v) => {
                                    val = Some(PyString::new_bound(
                                        py,
                                        v.as_str(),
                                    ))
                                }
                                Err(e) => {
                                    return Err(PyException::new_err(
                                        e.to_string(),
                                    ));
                                }
                            }

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

#[pymodule]
fn benda(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(switch, m)?)?;
    m.add_function(wrap_pyfunction!(load_book_from_file, m)?)?;
    m.add_function(wrap_pyfunction!(load_book, m)?)?;
    m.add_class::<BendCommand>()?;
    m.add_class::<PyBjit>()?;
    m.add_class::<U24>()?;
    m.add_class::<Tree>()?;
    m.add_class::<Node>()?;
    m.add_class::<Leaf>()?;
    m.add_class::<Fan>()?;
    Ok(())
}
