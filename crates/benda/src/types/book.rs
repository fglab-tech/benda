//! # Bend Integration Module
//!
//! This Rust module provides core functionality for integrating Bend, a specialized programming language, with Python.
//! It facilitates the execution of Bend code within a Python environment.
//!
//! ## Key Components
//!
//! - **Term Handling**: Structures and methods for working with Bend terms, including conversion to ADTs (Algebraic Data Types).
//! - **ADT Management**: Comprehensive support for Bend ADTs, including constructors and field management.
//! - **Function Definitions**: Structures for representing and executing Bend function definitions.
//! - **Runtime Options**: Enumeration of available runtimes (Rust, C, CUDA) for Bend code execution.
//! - **Book Representation**: A high-level structure (`Book`) that encapsulates ADTs and function definitions, serving as the main container for Bend-related data.
//! - **Python Integration**: Various utilities and methods to seamlessly interact with Bend constructs from Python code.
//!
//! ## Core Structures
//!
//! - `Term`: Represents HVM output in lambda encoding.
//! - `Ctrs`: Represents Bend ADTs with up to 8 constructors.
//! - `Definition` and `Definitions`: Handle individual and collections of Bend function definitions.
//! - `Adts`: Manages collections of Bend ADTs.
//! - `Book`: The primary structure holding all Bend-related components.
//!
//! ## Functionality
//!
//! - Convert between Bend lambda encoded terms and ADTs.
//! - Execute Bend functions with various runtime options.
//! - Manage and access Bend ADTs and function definitions.
//! - Provide a Pythonic interface to Bend constructs.
//!
//! This module forms the backbone of the Bend-Python integration, allowing developers to leverage Bend's capabilities within Python projects efficiently.

use core::panic;
use std::cell::RefCell;
use std::fmt::Display;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::vec;

use bend::fun::{self, Book as BendBook, Name, Rule};
use bend::imp::{self, Expr, Stmt};
use indexmap::IndexMap;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::types::{PyString, PyTuple};
use pyo3::PyTypeInfo;

use super::fan::Fan;
use super::user_adt::{from_term_into_adt, UserAdt};
use super::{extract_type_raw, BendType};
use crate::benda_ffi;
use crate::types::user_adt::BendCtr;

fn new_err<T>(str: String) -> PyResult<T> {
    Err(PyException::new_err(str))
}

thread_local!(static GLOBAL_BOOK: RefCell<Option<BendBook>> = const { RefCell::new(None) });
thread_local!(static GLOBAL_BENDA_BOOK: RefCell<Option<Book>> = const { RefCell::new(None) });

/// Term is the HVM output in lambda encoding.
///
/// This struct wraps a `bend::fun::Term` and provides methods to convert it to an ADT.
///
/// # Fields
///
/// * `term` - A `bend::fun::Term` representing the HVM output
#[pyclass(name = "Term")]
#[derive(Clone, Debug)]
pub struct Term {
    term: fun::Term,
}

#[pymethods]
impl Term {
    /// Returns a string representation of the Term
    ///
    /// This method uses the `display_pretty` function of the underlying `fun::Term`
    /// to generate a formatted string representation.
    ///
    /// # Returns
    ///
    /// A String containing the pretty-printed representation of the Term
    fn __str__(&self) -> String {
        self.term.display_pretty(0).to_string()
    }

    /// Converts the Term to an ADT (Algebraic Data Type)
    ///
    /// # Arguments
    ///
    /// * `t_type` - A PyAny object representing the target ADT type
    ///
    /// # Returns
    ///
    /// A PyResult containing the converted ADT as a PyAny object
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The conversion fails
    /// - An invalid type is provided
    /// - The Term cannot be parsed into the given ADT
    ///
    /// # Examples
    ///
    /// ```python
    /// term = some_function_returning_term()
    /// adt_type = MyADT  # Assuming MyADT is a defined Bend ADT
    /// result = term.to_adt(adt_type)
    /// ```
    fn to_adt(&self, t_type: Bound<PyAny>) -> PyResult<Py<PyAny>> {
        let py = t_type.py();

        let ctrs = t_type.downcast::<Ctrs>();

        match ctrs {
            Ok(ctrs) => {
                let adt = from_term_into_adt(
                    &self.term.clone(),
                    &ctrs.extract::<Ctrs>().unwrap(),
                );
                if let Some(adt) = adt {
                    match adt {
                        super::user_adt::TermParse::I32(val) => {
                            return Ok(val.into_py(py))
                        }
                        super::user_adt::TermParse::Any(any) => {
                            let list = any.extract::<Ctr2>(py);
                            return Ok(list.unwrap().into_py(py));
                        }
                        _ => {}
                    }
                };
                new_err("Could not parse Term into the given ADT".to_string())
            }
            Err(_) => new_err("Invalid Type given as argument".to_string()),
        }
    }
}

macro_rules! generate_structs {
    ($name:literal, $iden: ident) => {
        #[pyclass(name = $name)]
        #[derive(Clone, Debug)]
        pub(crate) struct $iden {
            full_name: String,
            fields: IndexMap<String, Option<Py<PyAny>>>,
        }

        impl BendCtr for $iden {
            fn to_py(&self, py: &Python) -> Py<PyAny> {
                Py::new(*py, self.clone()).unwrap().as_any().clone()
            }

            fn call_constructor(
                &mut self,
                args: Bound<PyTuple>,
            ) -> PyResult<PyObject> {
                self.__call__(args)
            }

            fn arity(&self) -> usize {
                self.fields.len()
            }
        }

        #[pymethods]
        impl $iden {
            #[classattr]
            fn __match_args__() -> PyResult<Py<PyAny>> {
                Python::with_gil(|py| {
                    Ok(PyTuple::new_bound(
                        py,
                        vec!["1", "2", "3", "4", "5", "6", "7", "8", "9", "10"],
                    )
                    .into_py(py))
                })
            }

            fn __str__(&self) -> String {
                let mut out = String::new();
                out.push_str(format!("<Bend ADT {}>", self.full_name).as_str());
                out
            }

            #[pyo3(signature = (*args))]
            pub fn __call__(
                &mut self,
                args: Bound<'_, PyTuple>,
            ) -> PyResult<PyObject> {
                let py = args.py();

                for (i, field) in self.fields.iter_mut().enumerate() {
                    field.1.replace(args.get_item(i).unwrap().to_object(py));
                }

                Ok(Py::new(py, self.clone()).unwrap().as_any().clone())
            }

            fn __setattr__(
                &mut self,
                field: Bound<PyAny>,
                value: Bound<PyAny>,
            ) {
                if let Some(val) = self.fields.get_mut(&field.to_string()) {
                    val.replace(value.to_object(field.py()));
                }
            }

            #[getter]
            fn r#type(&self) -> PyResult<PyObject> {
                Python::with_gil(|py| {
                    let ctr_type = $iden::type_object_bound(py).to_object(py);
                    Ok(ctr_type)
                })
            }

            fn __getattr__(&self, object: Bound<PyAny>) -> PyResult<PyObject> {
                let field = object.to_string();

                let py = object.py();

                if field == "__variant" {
                    return Ok(
                        PyString::new_bound(py, &self.full_name).into_py(py)
                    );
                }

                if field == "__ctr_type__" {
                    return Ok(
                        PyString::new_bound(py, &self.full_name).into_py(py)
                    );
                }

                if let Ok(val) = object.to_string().parse::<usize>() {
                    let return_val = self.fields.get_index(val - 1);
                    if let Some(return_val) = return_val {
                        return Ok(return_val.1.clone().into_py(py));
                    }
                }

                if let Some(val) = self.fields.get(&object.to_string()) {
                    Ok(val.clone().into_py(object.py()))
                } else {
                    new_err(format!("Could not find attr {}", object))
                }
            }
        }
    };
}

generate_structs!("Ctr1", Ctr1);
generate_structs!("Ctr2", Ctr2);
generate_structs!("Ctr3", Ctr3);
generate_structs!("Ctr4", Ctr4);
generate_structs!("Ctr5", Ctr5);
generate_structs!("Ctr6", Ctr6);
generate_structs!("Ctr7", Ctr7);
generate_structs!("Ctr8", Ctr8);

/// Represents a Bend ADT (Algebraic Data Type)
///
/// A Bend ADT is a collection of constructors, like: List, Tree, Map, etc.
/// This struct holds up to 8 constructors and provides methods to access them.
///
/// # Fields
///
/// * `fields` - An IndexMap of constructor names to their PyAny instance;
/// * `first` to `eighth` - Optional fields for up to 8 constructors (Ctr1 to Ctr8);
///
/// # Note
///
/// Due to pyo3 limitations, a Bend ADT used in Benda can have only up to 8 constructors.
/// This may change in the future if runtime types are added to pyo3.
///
/// # Examples
///
/// ```python
/// import benda
///
/// book = benda.load_book_from_file("./path/to/file.bend")
/// List = book.adts.List
///
/// a_list = List.Cons(1, List.Nil())
/// ```
#[pyclass(name = "Ctrs")]
#[derive(Clone, Debug, Default)]
pub struct Ctrs {
    fields: IndexMap<String, Py<PyAny>>,
    pub(crate) first: Option<Ctr1>,
    pub(crate) second: Option<Ctr2>,
    pub(crate) third: Option<Ctr3>,
    pub(crate) fourth: Option<Ctr4>,
    pub(crate) fifth: Option<Ctr5>,
    pub(crate) sixth: Option<Ctr6>,
    pub(crate) seventh: Option<Ctr7>,
    pub(crate) eighth: Option<Ctr8>,
}

impl Ctrs {
    /// Retrieves the base case constructor of the ADT, if it exists
    ///
    /// In Bend's lambda encoding, when a constructor has no fields (arity of 0),
    /// the HVM hides it when returning the Term. This method identifies such a constructor,
    /// which is considered the base case of the ADT.
    ///
    /// # Returns
    ///
    /// An `Option<Box<dyn BendCtr>>`:
    /// - `Some(Box<dyn BendCtr>)` if a base case constructor (arity 0) is found
    /// - `None` if no base case constructor is present in the ADT
    ///
    /// # Details
    ///
    /// The method checks each constructor (from `first` to `eighth`) in order.
    /// It returns the first constructor with an arity of 0, wrapped in a `Box<dyn BendCtr>`.
    ///
    /// # Note
    ///
    /// This method assumes that at least one of the constructors (up to `eighth`) is `Some`.
    /// It will panic if it encounters a `None` value before finding a base case or reaching the end.
    pub fn get_base_case(&self) -> Option<Box<dyn BendCtr>> {
        if self.first.as_ref().unwrap().arity() == 0 {
            return Some(Box::new(self.first.clone().unwrap()));
        }
        if self.second.as_ref().unwrap().arity() == 0 {
            return Some(Box::new(self.second.clone().unwrap()));
        }
        if self.third.as_ref().unwrap().arity() == 0 {
            return Some(Box::new(self.third.clone().unwrap()));
        }
        if self.fourth.as_ref().unwrap().arity() == 0 {
            return Some(Box::new(self.fourth.clone().unwrap()));
        }
        if self.fifth.as_ref().unwrap().arity() == 0 {
            return Some(Box::new(self.fifth.clone().unwrap()));
        }
        if self.sixth.as_ref().unwrap().arity() == 0 {
            return Some(Box::new(self.sixth.clone().unwrap()));
        }
        if self.seventh.as_ref().unwrap().arity() == 0 {
            return Some(Box::new(self.seventh.clone().unwrap()));
        }
        if self.eighth.as_ref().unwrap().arity() == 0 {
            return Some(Box::new(self.eighth.clone().unwrap()));
        }
        None
    }
}

#[pymethods]
impl Ctrs {
    fn __getattr__(&self, name: Bound<PyAny>) -> PyResult<pyo3::PyObject> {
        if let Some(val) = self.fields.get(&name.to_string()) {
            Ok(val.clone())
        } else {
            new_err(format!("Could not find attr {}", name))
        }
    }
}

/// Represents the available runtime options for executing Bend code
///
/// Bend supports three different runtimes: Rust, C, and CUDA. This enum allows
/// users to specify their preferred runtime for code execution.
///
/// # Variants
///
/// * `Rust` - The default Rust runtime
/// * `C` - The C runtime
/// * `Cuda` - The CUDA runtime (Note: not optimized for most video cards)
///
/// # Notes
///
/// - Rust is the default runtime and is generally recommended for most use cases.
/// - The C runtime is provided as an alternative option.
/// - The CUDA runtime is available but may not be optimized for the majority of video cards.
///
/// This enum implements `Clone`, `Debug`, and `Default` traits. The `Default` implementation
/// returns `BendRuntime::Rust`.
#[pyclass]
#[derive(Clone, Debug, Default)]
pub enum BendRuntime {
    #[default]
    Rust,
    C,
    Cuda,
}

impl Display for BendRuntime {
    /// Implements the `Display` trait for `BendRuntime`
    ///
    /// This implementation allows `BendRuntime` variants to be converted into
    /// string representations that correspond to the actual command used to run
    /// the Bend code with the specified runtime.
    ///
    /// # Returns
    ///
    /// A `String` representing the command for the chosen runtime:
    /// - `"run"` for `BendRuntime::Rust`
    /// - `"run-c"` for `BendRuntime::C`
    /// - `"run-cu"` for `BendRuntime::Cuda`
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BendRuntime::Rust => f.write_str("run"),
            BendRuntime::C => f.write_str("run-c"),
            BendRuntime::Cuda => f.write_str("run-cu"),
        }
    }
}

/// Represents a Bend function definition
///
/// This struct holds information about a Bend function, including its name, arity,
/// and the runtime command to be used for execution.
///
/// # Fields
///
/// * `arity` - The number of arguments the function expects
/// * `name` - The name of the function
/// * `cmd` - An optional `BendRuntime` specifying the runtime to use for execution
#[pyclass(name = "Definition")]
#[derive(Clone, Debug, Default)]
pub struct Definition {
    arity: usize,
    name: String,
    cmd: Option<BendRuntime>,
}

#[pymethods]
impl Definition {
    /// Returns a string representation of the Definition
    ///
    /// # Returns
    ///
    /// A string in the format "Bend function: name(arity)"
    fn __str__(&self) -> String {
        format!("Bend function: {}({})", self.name, self.arity)
    }

    /// Calls the Bend function with the given arguments
    ///
    /// This method executes the Bend function, handling argument processing,
    /// function call setup, and result parsing.
    ///
    /// # Arguments
    ///
    /// * `args` - A tuple of Python arguments passed to the function;
    ///
    /// # Returns
    ///
    /// A `Term` containing the result of the function execution, that can be parsed into a ADT
    /// using `to_adt()` method.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The number of arguments doesn't match the function's arity
    /// - The global Bend book is not available
    /// - The HVM output cannot be parsed
    /// - The function execution fails for any reason
    #[pyo3(signature = (*args))]
    fn __call__(&mut self, args: Bound<'_, PyTuple>) -> PyResult<Py<PyAny>> {
        let py = args.py();

        let bend_book = GLOBAL_BOOK.take();

        if self.arity != args.len() && self.arity != 0 {
            return new_err(format!(
                "Function has arity {} and received {} arguments",
                self.arity,
                args.len(),
            ));
        }

        let mut new_args: Vec<Expr> = vec![];

        if let Some(mut b) = bend_book.clone() {
            for (arg_num, arg) in args.iter().enumerate() {
                let arg_name = Name::new(format!("arg{}", arg_num));

                let mut u_type: Option<fun::Term> = None;

                if let Ok(term) = arg.downcast::<Term>() {
                    if let Ok(new_term) = term.extract::<Term>() {
                        u_type = Some(new_term.term);
                    }
                } else if let Ok(term) = arg.downcast::<Fan>() {
                    if let Ok(new_term) = term.extract::<Fan>() {
                        u_type = Some(new_term.term);
                    }
                } else {
                    let adt = UserAdt::new(arg.clone(), &b);

                    let new_arg: Expr;

                    if let Some(adt) = adt {
                        new_arg = adt.to_bend().unwrap();
                    } else {
                        new_arg = extract_type_raw(arg.clone())
                            .unwrap()
                            .to_bend()
                            .unwrap();
                    }

                    u_type = Some(new_arg.clone().to_fun());
                }

                if let Some(n_type) = u_type {
                    let def = fun::Definition {
                        name: arg_name.clone(),
                        rules: vec![Rule {
                            pats: vec![],
                            body: n_type,
                        }],
                        builtin: false,
                    };

                    b.defs.insert(arg_name.clone(), def);

                    new_args.push(Expr::Var {
                        nam: arg_name.clone(),
                    });
                }
            }

            let first = Stmt::Return {
                term: Box::new(Expr::Call {
                    fun: Box::new(imp::Expr::Var {
                        nam: Name::new(self.name.to_string()),
                    }),
                    args: new_args,
                    kwargs: vec![],
                }),
            };

            let main_def = imp::Definition {
                name: Name::new("main"),
                params: vec![],
                body: first,
            };

            b.defs
                .insert(Name::new("main"), main_def.to_fun(true).unwrap());

            let res = benda_ffi::run(
                &b,
                &self.cmd.clone().unwrap_or_default().to_string(),
            );

            GLOBAL_BOOK.set(bend_book);

            let ret_term: Term;

            match res {
                Ok(res) => match res {
                    Some(res) => ret_term = Term { term: res.0 },
                    None => {
                        return new_err(
                            "Could not parse HVM output".to_string(),
                        )
                    }
                },
                Err(e) => return new_err(e.to_string()),
            }

            return Ok(ret_term.into_py(py));
        }

        new_err(format!("Could not execute function {}", self.name))
    }
}

/// Represents a collection of Bend function definitions
///
/// This struct holds multiple Bend function definitions and provides a way to
/// retrieve them by name.
///
/// # Fields
///
/// * `defs` - An IndexMap of function names to their corresponding `Definition`s;
/// * `cmd` - An optional `BendRuntime` specifying the preferred runtime for all definitions;
#[pyclass(name = "Definitions")]
#[derive(Clone, Debug, Default)]
pub struct Definitions {
    defs: IndexMap<String, Definition>,
    cmd: Option<BendRuntime>,
}

#[pymethods]
impl Definitions {
    /// Retrieves a Definition by its name
    ///
    /// This method allows accessing individual function definitions as attributes
    /// of the Definitions object.
    ///
    /// # Arguments
    ///
    /// * `object` - A `Bound<PyAny>` representing the attribute name (function name);
    ///
    /// # Returns
    ///
    /// A `Definition` containing the requested Definition
    ///
    /// # Errors
    ///
    /// Returns an error if the requested function definition is not found
    fn __getattr__(&self, object: Bound<PyAny>) -> PyResult<Py<Definition>> {
        let binding = object.to_string();
        let field = binding.as_str();
        let py = object.py();

        if let Some(def) = self.defs.get(field) {
            let mut def = def.clone();
            def.cmd = self.cmd.clone();
            Ok(Py::new(py, def)?)
        } else {
            new_err(format!("Could not find attr {}", object))
        }
    }
}

/// Represents a collection of Algebraic Data Types (ADTs) in Bend
///
/// This struct holds multiple ADTs, each represented by a `Ctrs` object.
#[pyclass(name = "Adt")]
#[derive(Clone, Debug)]
pub struct Adts {
    adts: IndexMap<String, Ctrs>,
}

impl Adts {
    /// Creates a new, empty Adts collection
    ///
    /// # Returns
    ///
    /// A new `Adts` instance with an empty `IndexMap`
    fn new() -> Self {
        Self {
            adts: IndexMap::new(),
        }
    }
}

#[pymethods]
impl Adts {
    /// Retrieves an ADT by its name
    ///
    /// # Arguments
    ///
    /// * `object` - A `Bound<PyAny>` representing the ADT name
    ///
    /// # Returns
    ///
    /// A `PyResult<PyObject>` containing the requested ADT as a Python object
    ///
    /// # Errors
    ///
    /// Returns an error if the requested ADT is not found
    fn __getattr__(&self, object: Bound<PyAny>) -> PyResult<PyObject> {
        let binding = object.to_string();
        let field = binding.as_str();
        let py = object.py();

        if let Some(ctrs) = self.adts.get(field) {
            Ok(ctrs.clone().into_py(py))
        } else {
            new_err(format!("Could not find attr {}", object))
        }
    }
}

/// Represents a Bend Book, containing ADTs and function definitions
///
/// This struct is the main container for all Bend-related data, including
/// Algebraic Data Types (ADTs) and function definitions.
#[pyclass(name = "Book")]
#[derive(Clone, Debug)]
pub struct Book {
    adts: Adts,
    defs: Definitions,
    cmd: Option<BendRuntime>,
}

impl Book {
    /// Creates a new Book from a Bend Book
    ///
    /// This method initializes a Book struct with ADTs and function definitions
    /// from a Bend Book. It also sets up global state for the Bend Book and the created Book.
    ///
    /// # Arguments
    ///
    /// * `bend_book` - A mutable reference to a Bend Book
    ///
    /// # Returns
    ///
    /// A new `Book` instance
    pub fn new(bend_book: &mut BendBook) -> Self {
        let mut adts = Adts::new();

        for (adt_name, bend_adt) in bend_book.adts.iter() {
            let mut all_ctrs = Ctrs::default();

            let mut first: Option<Ctr1> = None;
            let mut second: Option<Ctr2> = None;
            let mut third: Option<Ctr3> = None;
            let mut fourth: Option<Ctr4> = None;
            let mut fifth: Option<Ctr5> = None;
            let mut sixth: Option<Ctr6> = None;
            let mut seventh: Option<Ctr7> = None;
            let mut eighth: Option<Ctr8> = None;

            for (index, (ctr_name, ctr_fields)) in
                bend_adt.ctrs.iter().enumerate()
            {
                let new_name = ctr_name.split('/').last().unwrap().to_string();

                Python::with_gil(|py| match index {
                    0 => {
                        let mut ct = Ctr1 {
                            full_name: ctr_name.to_string(),
                            fields: IndexMap::new(),
                        };
                        for c in ctr_fields {
                            ct.fields.insert(c.nam.to_string(), None);
                        }
                        first = Some(ct.clone());
                        all_ctrs
                            .fields
                            .insert(new_name, ct.into_py(py).as_any().clone());
                    }
                    1 => {
                        let mut ct = Ctr2 {
                            full_name: ctr_name.to_string(),
                            fields: IndexMap::new(),
                        };
                        for c in ctr_fields {
                            ct.fields.insert(c.nam.to_string(), None);
                        }
                        second = Some(ct.clone());
                        all_ctrs
                            .fields
                            .insert(new_name, ct.into_py(py).as_any().clone());
                    }
                    2 => {
                        let mut ct = Ctr3 {
                            full_name: ctr_name.to_string(),
                            fields: IndexMap::new(),
                        };
                        for c in ctr_fields {
                            ct.fields.insert(c.nam.to_string(), None);
                        }
                        third = Some(ct.clone());
                        all_ctrs
                            .fields
                            .insert(new_name, ct.into_py(py).as_any().clone());
                    }
                    3 => {
                        let mut ct = Ctr4 {
                            full_name: ctr_name.to_string(),
                            fields: IndexMap::new(),
                        };
                        for c in ctr_fields {
                            ct.fields.insert(c.nam.to_string(), None);
                        }
                        fourth = Some(ct.clone());
                        all_ctrs
                            .fields
                            .insert(new_name, ct.into_py(py).as_any().clone());
                    }

                    4 => {
                        let mut ct = Ctr5 {
                            full_name: ctr_name.to_string(),
                            fields: IndexMap::new(),
                        };
                        for c in ctr_fields {
                            ct.fields.insert(c.nam.to_string(), None);
                        }
                        fifth = Some(ct.clone());
                        all_ctrs
                            .fields
                            .insert(new_name, ct.into_py(py).as_any().clone());
                    }

                    5 => {
                        let mut ct = Ctr6 {
                            full_name: ctr_name.to_string(),
                            fields: IndexMap::new(),
                        };
                        for c in ctr_fields {
                            ct.fields.insert(c.nam.to_string(), None);
                        }
                        sixth = Some(ct.clone());
                        all_ctrs
                            .fields
                            .insert(new_name, ct.into_py(py).as_any().clone());
                    }
                    6 => {
                        let mut ct = Ctr7 {
                            full_name: ctr_name.to_string(),
                            fields: IndexMap::new(),
                        };
                        for c in ctr_fields {
                            ct.fields.insert(c.nam.to_string(), None);
                        }
                        seventh = Some(ct.clone());
                        all_ctrs
                            .fields
                            .insert(new_name, ct.into_py(py).as_any().clone());
                    }

                    7 => {
                        let mut ct = Ctr8 {
                            full_name: ctr_name.to_string(),
                            fields: IndexMap::new(),
                        };
                        for c in ctr_fields {
                            ct.fields.insert(c.nam.to_string(), None);
                        }
                        eighth = Some(ct.clone());
                        all_ctrs
                            .fields
                            .insert(new_name, ct.into_py(py).as_any().clone());
                    }
                    _ => panic!("Type must have up to 5 Ctrs"),
                });
            }

            all_ctrs.first = first;
            all_ctrs.second = second;
            all_ctrs.third = third;
            all_ctrs.fourth = fourth;
            all_ctrs.fifth = fifth;
            all_ctrs.sixth = sixth;
            all_ctrs.seventh = seventh;
            all_ctrs.eighth = eighth;

            adts.adts.insert(adt_name.to_string(), all_ctrs);
        }

        let mut definitions = Definitions::default();

        for (nam, def) in bend_book.defs.iter() {
            let new_def = Definition {
                arity: def.arity(),
                name: def.name.to_string(),
                cmd: None,
            };
            definitions.defs.insert(nam.to_string(), new_def);
        }

        bend_book.defs.shift_remove(&Name::new("Main"));
        bend_book.defs.shift_remove(&Name::new("main"));

        GLOBAL_BOOK.set(Some(bend_book.clone()));

        let benda_book = Self {
            adts,
            defs: definitions,
            cmd: None,
        };
        GLOBAL_BENDA_BOOK.set(Some(benda_book.clone()));

        benda_book
    }
}

#[pymethods]
impl Book {
    /// Sets the runtime command for the Book
    ///
    /// # Arguments
    ///
    /// * `cmd` - A `BendRuntime` specifying the runtime to use
    fn set_cmd(&mut self, cmd: BendRuntime) {
        self.cmd = Some(cmd);
    }

    /// Retrieves the ADTs contained in the Book
    ///
    /// # Returns
    ///
    /// A `PyResult<PyObject>` containing the ADTs as a Python object
    #[getter]
    fn adts(&self) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            let adt = &self.adts;
            Ok(adt.clone().into_py(py))
        })
    }

    /// Retrieves the function definitions contained in the Book
    ///
    /// # Returns
    ///
    /// A `PyResult<PyObject>` containing the function definitions as a Python object
    #[getter]
    fn defs(&self) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            let mut defs = self.defs.clone();
            defs.cmd = self.cmd.clone();
            Ok(defs.into_py(py))
        })
    }

    /// Fallback method for attribute access
    ///
    /// # Arguments
    ///
    /// * `attr_name` - A `Bound<PyAny>` representing the attribute name
    ///
    /// # Returns
    ///
    /// A `PyResult<PyObject>`
    ///
    /// # Errors
    ///
    /// Always returns an error, as this is a fallback method for non-existent attributes
    fn __getattr__(&self, attr_name: Bound<PyAny>) -> PyResult<PyObject> {
        let attr_name = attr_name.to_string();

        new_err(format!("Could not find attribute {}", attr_name))
    }
}
