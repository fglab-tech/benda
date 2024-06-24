use core::panic;
use std::cell::RefCell;
use std::vec;

use bend::fun::{self, Book as BendBook, Name, Rule};
use bend::imp::{self, Expr, Stmt};
use indexmap::IndexMap;
use num_traits::ToPrimitive;
use pyo3::exceptions::PyException;
use pyo3::ffi::{PyMethodDef, PyType_FromSpec, PyType_IsSubtype};
use pyo3::inspect::types::{self, ModuleName, TypeInfo};
use pyo3::prelude::*;
use pyo3::types::{PyList, PyMapping, PyString, PyTuple, PyType};
use pyo3::{type_object, PyClass, PyTypeCheck, PyTypeInfo};

use super::user_adt::UserAdt;
use super::{extract_type_raw, BendType};
use crate::benda_ffi;

fn new_err<T>(str: String) -> PyResult<T> {
    Err(PyException::new_err(str))
}

thread_local!(static GLOBAL_BOOK: RefCell<Option<BendBook>> = const { RefCell::new(None) });

#[pyclass(name = "Term")]
#[derive(Clone, Debug)]
pub struct Term {
    term: fun::Term,
}

fn get_list(lam: &fun::Term, vals: &mut Vec<i32>) {
    match lam {
        fun::Term::Lam { tag, pat, bod } => {
            get_list(bod, vals);
        }
        fun::Term::App { tag, fun, arg } => {
            get_list(fun, vals);
            if let fun::Term::Num { val } = **arg {
                match val {
                    fun::Num::U24(v) => vals.push(v.to_i32().unwrap()),
                    fun::Num::I24(v) => vals.push(v.to_i32().unwrap()),
                    fun::Num::F24(v) => vals.push(v.to_i32().unwrap()),
                }
            }
        }
        _ => {}
    }
}

#[pymethods]
impl Term {
    fn __str__(&self) -> String {
        self.term.to_string()
    }

    fn __getattr__(&self, object: Bound<PyAny>) -> PyResult<PyObject> {
        let py = object.py();

        let mut vals: Vec<i32> = vec![];
        get_list(&self.term, &mut vals);

        Ok(vals.into_py(py))
    }
}

#[pyclass(name = "Ctr2")]
#[derive(Clone, Debug)]
pub struct Ctr2 {
    entire_name: String,
    name: String,
    fields: IndexMap<String, Option<Py<PyAny>>>,
}

#[pymethods]
impl Ctr2 {
    #[classattr]
    fn __match_args__() -> PyResult<Py<PyAny>> {
        Python::with_gil(|py| {
            Ok(PyTuple::new_bound(py, vec!["1", "2", "3", "4", "5"])
                .into_py(py))
        })
    }

    fn __str__(&self) -> String {
        format!("Bend ADT: {}", self.entire_name)
    }

    #[pyo3(signature = (*args))]
    fn __call__(&mut self, args: Bound<'_, PyTuple>) -> PyResult<PyObject> {
        let py = args.py();

        for (i, field) in self.fields.iter_mut().enumerate() {
            field.1.replace(args.get_item(i).unwrap().to_object(py));
        }

        Ok(Py::new(py, self.clone()).unwrap().as_any().clone())
    }

    fn __setattr__(&mut self, field: Bound<PyAny>, value: Bound<PyAny>) {
        if let Some(val) = self.fields.get_mut(&field.to_string()) {
            val.replace(value.to_object(field.py()));
        }
    }

    fn __getattr__(&self, object: Bound<PyAny>) -> PyResult<PyObject> {
        let field = object.to_string();

        let py = object.py();

        if field == "type" {
            return Ok(PyString::new_bound(py, &self.entire_name).into_py(py));
        }

        if &object.to_string() == "name" {
            return Ok(PyString::new_bound(py, &self.name).into());
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

#[pyclass(name = "Ctr")]
#[derive(Clone, Debug)]
pub struct Ctr {
    entire_name: String,
    name: String,
    fields: IndexMap<String, Option<Py<PyAny>>>,
}

#[pymethods]
impl Ctr {
    #[classattr]
    fn __match_args__() -> PyResult<Py<PyAny>> {
        Python::with_gil(|py| {
            Ok(PyTuple::new_bound(py, vec!["1", "2", "3", "4", "5"])
                .into_py(py))
        })
    }

    fn __str__(&self) -> String {
        format!("Bend ADT: {}", self.entire_name)
    }

    #[pyo3(signature = (*args))]
    fn __call__(&mut self, args: Bound<'_, PyTuple>) -> PyResult<PyObject> {
        let py = args.py();

        for (i, field) in self.fields.iter_mut().enumerate() {
            field.1.replace(args.get_item(i).unwrap().to_object(py));
        }

        Ok(Py::new(py, self.clone()).unwrap().as_any().clone())
    }

    fn __setattr__(&mut self, field: Bound<PyAny>, value: Bound<PyAny>) {
        if let Some(val) = self.fields.get_mut(&field.to_string()) {
            val.replace(value.to_object(field.py()));
        }
    }

    fn __getattr__(&self, object: Bound<PyAny>) -> PyResult<PyObject> {
        let field = object.to_string();

        let py = object.py();

        if field == "type" {
            return Ok(PyString::new_bound(py, &self.entire_name).into_py(py));
        }

        if &object.to_string() == "name" {
            return Ok(PyString::new_bound(py, &self.name).into());
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

#[pyclass(name = "Ctrs")]
#[derive(Clone, Debug, Default)]
pub struct Ctrs {
    fields: IndexMap<String, Py<PyAny>>,
    first: Option<Ctr>,
    second: Option<Ctr2>,
}

#[pymethods]
impl Ctrs {
    fn __getattr__(&self, object: Bound<PyAny>) -> PyResult<pyo3::PyObject> {
        let py = object.py();

        if object.to_string().starts_with("t") {
            let b_name = object.to_string();
            let name = b_name.strip_prefix("t").unwrap();

            let index = self.fields.get_index_of(name).unwrap();

            let res = match index {
                0 => Ctr::type_object_bound(py),
                1 => Ctr2::type_object_bound(py),
                _ => {
                    return new_err(
                        "Type can only have up to 2 constructors".to_string(),
                    )
                }
            };

            return Ok(res.to_object(py));
        }

        if let Some(val) = self.fields.get(&object.to_string()) {
            Ok(val.clone())
        } else {
            new_err(format!("Could not find attr {}", object))
        }
    }
}

#[pyclass(name = "Definition")]
#[derive(Clone, Debug, Default)]
pub struct Definition {
    arity: usize,
    name: String,
}

#[pymethods]
impl Definition {
    fn __str__(&self) -> String {
        format!("Bend function: {}({})", self.name, self.arity)
    }

    #[pyo3(signature = (*args))]
    fn __call__(&mut self, args: Bound<'_, PyTuple>) -> PyResult<Py<PyAny>> {
        let py = args.py();

        let bend_book = GLOBAL_BOOK.take();

        if self.arity != args.len() {
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

            let res = benda_ffi::run(&b.clone());

            GLOBAL_BOOK.set(bend_book);

            let ret_term = Term {
                term: res.unwrap().0,
            };

            return Ok(ret_term.into_py(py));
        }

        new_err(format!("Could not execute function {}", self.name))
    }
}

#[pyclass(name = "Definitions")]
#[derive(Clone, Debug, Default)]
pub struct Definitions {
    defs: IndexMap<String, Definition>,
}

#[pymethods]
impl Definitions {
    fn __getattr__(&self, object: Bound<PyAny>) -> PyResult<Py<Definition>> {
        let binding = object.to_string();
        let field = binding.as_str();
        let py = object.py();

        if let Some(def) = self.defs.get(field) {
            Ok(Py::new(py, def.clone())?)
        } else {
            new_err(format!("Could not find attr {}", object))
        }
    }
}

#[pyclass(name = "Adt")]
#[derive(Clone, Debug)]
pub struct Adts {
    adts: IndexMap<String, Ctrs>,
}

impl Adts {
    fn new() -> Self {
        Self {
            adts: IndexMap::new(),
        }
    }
}

#[pymethods]
impl Adts {
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

#[pyclass(name = "Book")]
pub struct Book {
    adts: Adts,
    defs: Definitions,
}

impl Book {
    pub fn new(bend_book: BendBook) -> Self {
        let mut adts = Adts::new();

        for (adt_name, bend_adt) in bend_book.adts.iter() {
            let mut all_ctrs = Ctrs::default();

            let mut first: Option<Ctr> = None;
            let mut second: Option<Ctr2> = None;

            for (index, (ctr_name, ctr_fields)) in
                bend_adt.ctrs.iter().enumerate()
            {
                let new_name = ctr_name.split('/').last().unwrap().to_string();

                Python::with_gil(|py| match index {
                    0 => {
                        let mut ct = Ctr {
                            name: new_name.clone(),
                            entire_name: ctr_name.to_string(),
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
                            name: new_name.clone(),
                            entire_name: ctr_name.to_string(),
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
                    _ => panic!("Type must have only 2 Ctrs"),
                });
            }

            all_ctrs.first = first;
            all_ctrs.second = second;

            adts.adts.insert(adt_name.to_string(), all_ctrs);
        }

        let mut definitions = Definitions::default();

        for (nam, def) in bend_book.defs.iter() {
            let new_def = Definition {
                arity: def.arity(),
                name: def.name.to_string(),
            };
            definitions.defs.insert(nam.to_string(), new_def);
        }

        GLOBAL_BOOK.set(Some(bend_book.clone()));

        Self {
            adts,
            defs: definitions,
        }
    }
}

#[pymethods]
impl Book {
    fn __getattr__(&self, object: Bound<PyAny>) -> PyResult<PyObject> {
        let binding = object.to_string();
        let field = binding.as_str();
        let py = object.py();

        match field {
            "adts" => {
                let adt = &self.adts;
                Ok(adt.clone().into_py(py))
            }
            "defs" => {
                let def = &self.defs;
                Ok(def.clone().into_py(py))
            }

            _ => new_err(format!("Could not find attribute {}", object)),
        }
    }
}
