use core::panic;
use std::cell::RefCell;
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

/// Term is the HVM output in lambda encoding. Use `to_adt` method to turn it into a ADT.
#[pyclass(name = "Term")]
#[derive(Clone, Debug)]
pub struct Term {
    term: fun::Term,
}

#[pymethods]
impl Term {
    fn __str__(&self) -> String {
        self.term.display_pretty(0).to_string()
    }

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
                            println!("val {}", val)
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
        pub struct $iden {
            entire_name: String,
            name: String,
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
                    Ok(PyTuple::new_bound(py, vec!["1", "2", "3", "4", "5"])
                        .into_py(py))
                })
            }

            fn __str__(&self) -> String {
                format!("Bend ADT: {}", self.entire_name)
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

            fn __getattr__(&self, object: Bound<PyAny>) -> PyResult<PyObject> {
                let field = object.to_string();

                let py = object.py();

                if field == "type" {
                    return Ok(
                        PyString::new_bound(py, &self.entire_name).into_py(py)
                    );
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
    };
}

generate_structs!("Ctr1", Ctr1);
generate_structs!("Ctr2", Ctr2);
generate_structs!("Ctr3", Ctr3);
generate_structs!("Ctr4", Ctr4);
generate_structs!("Ctr5", Ctr5);

#[pyclass(name = "Ctrs")]
#[derive(Clone, Debug, Default)]
pub struct Ctrs {
    fields: IndexMap<String, Py<PyAny>>,
    pub first: Option<Ctr1>,
    pub second: Option<Ctr2>,
    pub third: Option<Ctr3>,
    pub fourth: Option<Ctr4>,
    pub fifth: Option<Ctr5>,
}

impl Ctrs {
    pub(crate) fn get_base_case(&self) -> Option<Box<dyn BendCtr>> {
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
        None
    }
}

#[pymethods]
impl Ctrs {
    fn __getattr__(&self, object: Bound<PyAny>) -> PyResult<pyo3::PyObject> {
        let py = object.py();

        if object.to_string().starts_with('t') {
            let b_name = object.to_string();
            let name = b_name.strip_prefix('t').unwrap();

            let index = self.fields.get_index_of(name).unwrap();

            let res = match index {
                0 => Ctr1::type_object_bound(py),
                1 => Ctr2::type_object_bound(py),
                2 => Ctr3::type_object_bound(py),
                3 => Ctr4::type_object_bound(py),
                4 => Ctr5::type_object_bound(py),
                _ => {
                    return new_err(
                        "Type can only have up to 5 constructors".to_string(),
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
#[derive(Clone, Debug)]
pub struct Book {
    adts: Adts,
    defs: Definitions,
}

impl Book {
    pub fn new(bend_book: &mut BendBook) -> Self {
        let mut adts = Adts::new();

        for (adt_name, bend_adt) in bend_book.adts.iter() {
            let mut all_ctrs = Ctrs::default();

            let mut first: Option<Ctr1> = None;
            let mut second: Option<Ctr2> = None;
            let mut third: Option<Ctr3> = None;
            let mut fourth: Option<Ctr4> = None;
            let mut fifth: Option<Ctr5> = None;

            for (index, (ctr_name, ctr_fields)) in
                bend_adt.ctrs.iter().enumerate()
            {
                let new_name = ctr_name.split('/').last().unwrap().to_string();

                Python::with_gil(|py| match index {
                    0 => {
                        let mut ct = Ctr1 {
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
                    2 => {
                        let mut ct = Ctr3 {
                            name: new_name.clone(),
                            entire_name: ctr_name.to_string(),
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
                            name: new_name.clone(),
                            entire_name: ctr_name.to_string(),
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
                            name: new_name.clone(),
                            entire_name: ctr_name.to_string(),
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

                    _ => panic!("Type must have up to 5 Ctrs"),
                });
            }

            all_ctrs.first = first;
            all_ctrs.second = second;
            all_ctrs.third = third;
            all_ctrs.fourth = fourth;
            all_ctrs.fifth = fifth;

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

        bend_book.defs.shift_remove(&Name::new("Main"));
        bend_book.defs.shift_remove(&Name::new("main"));

        GLOBAL_BOOK.set(Some(bend_book.clone()));

        let benda_book = Self {
            adts,
            defs: definitions,
        };
        GLOBAL_BENDA_BOOK.set(Some(benda_book.clone()));

        benda_book
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
