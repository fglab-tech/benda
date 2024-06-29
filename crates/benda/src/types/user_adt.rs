use std::vec;

use bend::fun::{Adt as BAdt, Book, Name, Num, Term as BTerm};
use bend::imp::{self};
use num_traits::ToPrimitive;
use pyo3::types::{PyAnyMethods, PyString, PyTuple};
use pyo3::{Bound, IntoPy, Py, PyAny, PyErr, PyObject, PyResult, Python};

use super::book::Ctrs;
use super::{extract_type_raw, BendType};

fn num_to_i32(num: &Num) -> i32 {
    match num {
        Num::U24(val) => val.to_i32().unwrap(),
        Num::I24(val) => *val,
        Num::F24(val) => val.to_i32().unwrap(),
    }
}

#[derive(Debug)]
pub enum TermParse {
    I32(i32),
    Ctr(Box<dyn BendCtr>),
    Any(Py<PyAny>),
    Vec(Box<dyn BendCtr>, Vec<Py<PyAny>>),
    Args(Vec<Py<PyAny>>),
}

pub(crate) trait BendCtr: std::fmt::Debug {
    fn to_py(&self, py: &Python) -> Py<PyAny>;
    fn call_constructor(&mut self, args: Bound<PyTuple>) -> PyResult<PyObject>;
    fn arity(&self) -> usize;
}

pub fn from_term_into_adt(term: &BTerm, def_adts: &Ctrs) -> Option<TermParse> {
    match term {
        BTerm::Lam {
            tag: _,
            pat: _,
            bod,
        } => {
            let mut args: Vec<Py<PyAny>> = vec![];

            let lam_body = from_term_into_adt(bod.as_ref(), def_adts);

            if let Some(bod) = lam_body {
                match bod {
                    TermParse::I32(val) => return Some(TermParse::I32(val)),
                    TermParse::Ctr(mut ct) => {
                        if ct.arity() == 0 {
                            return Python::with_gil(|py| {
                                return Some(TermParse::Any(
                                    ct.call_constructor(PyTuple::empty_bound(
                                        py,
                                    ))
                                    .unwrap(),
                                ));
                            });
                        }
                    }
                    TermParse::Any(a) => {
                        args.push(a);
                    }
                    TermParse::Vec(mut ct, mut args) => {
                        return Python::with_gil(|py| {
                            if let Some(case) = def_adts.get_base_case() {
                                args.push(case.to_py(&py));
                            }

                            return Some(TermParse::Any(
                                ct.call_constructor(PyTuple::new_bound(
                                    py, args,
                                ))
                                .unwrap(),
                            ));
                        });
                    }
                    TermParse::Args(_) => {
                        todo!()
                    }
                };
            }
            todo!()
        }
        BTerm::App { tag: _, fun, arg } => {
            if let (BTerm::Var { nam: _ }, BTerm::Num { val }) =
                (fun.as_ref(), arg.as_ref())
            {
                let constructor: Option<Box<dyn BendCtr>> =
                    match num_to_i32(val) {
                        0 => Some(Box::new(def_adts.first.clone().unwrap())),
                        1 => Some(Box::new(def_adts.second.clone().unwrap())),
                        2 => Some(Box::new(def_adts.third.clone().unwrap())),
                        3 => Some(Box::new(def_adts.fourth.clone().unwrap())),
                        4 => Some(Box::new(def_adts.fifth.clone().unwrap())),
                        _ => panic!("ADT has more than 5 Ctrs"),
                    };

                return Some(TermParse::Ctr(constructor.unwrap()));
            }

            let app_arg = from_term_into_adt(arg, def_adts);
            let app_fun = from_term_into_adt(fun, def_adts);

            let mut args: Vec<Py<PyAny>> = vec![];

            if let Some(app_arg) = app_arg {
                match app_arg {
                    TermParse::I32(val) => {
                        Python::with_gil(|py| args.push(val.into_py(py)));
                    }
                    TermParse::Ctr(_) => todo!(),
                    TermParse::Any(a) => args.push(a),
                    TermParse::Vec(_, _) => todo!(),
                    TermParse::Args(mut inner_args) => {
                        args.append(&mut inner_args);
                    }
                }
            }

            if let Some(a_fun) = app_fun {
                match a_fun {
                    TermParse::I32(_) => {}
                    TermParse::Ctr(c) => {
                        return Some(TermParse::Vec(c, args));
                    }
                    TermParse::Any(a) => {
                        args.push(a);
                    }
                    TermParse::Vec(ct, mut ct_args) => {
                        ct_args.append(&mut args);
                        return Some(TermParse::Vec(ct, ct_args));
                    }
                    TermParse::Args(mut inner_args) => {
                        args.append(&mut inner_args);
                    }
                }
            }
            Some(TermParse::Args(args))
        }
        BTerm::Num { val } => Some(TermParse::I32(num_to_i32(val))),
        _ => None,
    }
}
pub struct UserAdt<'py> {
    adt: BAdt,
    entire_nam: Name,
    data: Bound<'py, PyAny>,
    book: Book,
}

impl<'py> UserAdt<'py> {
    pub fn new(data: Bound<'py, PyAny>, book: &Book) -> Option<Self> {
        if data.is_none() {
            return None;
        }

        // TODO: make check for every Ctr
        //if data.clone().get_type().qualname().unwrap() != "Ctr" {
        //    return None;
        //}

        if let Ok(binding) = data.getattr("type") {
            for (nam, _ctr) in &book.ctrs {
                let new_nam = nam.to_string();
                let two_names = new_nam.split_once('/').unwrap();

                if nam.to_string() == binding.to_string() {
                    return Some(Self {
                        book: book.clone(),
                        data,
                        entire_nam: Name::new(new_nam.clone()),
                        adt: book
                            .adts
                            .get(&Name::new(two_names.0.to_string()))
                            .unwrap()
                            .clone(),
                    });
                }
            }
        }

        None
    }
}

impl<'py> BendType for UserAdt<'py> {
    fn to_bend(&self) -> super::ToBendResult {
        for (nam, fields) in &self.adt.ctrs {
            if *nam == self.entire_nam {
                let mut adt_fields: Vec<imp::Expr> = vec![];

                for field in fields {
                    let attr_nam = field.nam.clone();

                    let py = self.data.py();

                    let attr = self
                        .data
                        .getattr(PyString::new_bound(py, attr_nam.as_ref()))
                        .unwrap();

                    if let Some(t) = extract_type_raw(attr.clone()) {
                        adt_fields.push(t.to_bend().unwrap());
                    } else if let Some(adt) = UserAdt::new(attr, &self.book) {
                        let new_adt = adt.to_bend();
                        adt_fields.push(new_adt.unwrap());
                    } else {
                        let field_name = nam.split('/').nth(0).unwrap();

                        let new_ctr = self
                            .book
                            .adts
                            .get(&Name::new(field_name.to_string()));

                        for c in new_ctr.unwrap().ctrs.iter() {
                            if c.1.is_empty() {
                                adt_fields.push(imp::Expr::Ctr {
                                    name: c.0.clone(),
                                    args: vec![],
                                    kwargs: vec![],
                                })
                            }
                        }
                    }
                }

                return Ok(imp::Expr::Ctr {
                    name: nam.clone(),
                    args: adt_fields,
                    kwargs: vec![],
                });
            }
        }
        Err(PyErr::fetch(self.data.py()))
    }
}
