use bend::fun::{Adt as BAdt, Book, Name, Num, Term as BTerm};
use bend::imp::{self};
use num_traits::ToPrimitive;
use pyo3::types::{PyAnyMethods, PyString, PyTuple};
use pyo3::{Bound, IntoPy, Py, PyAny, PyErr, PyObject, Python};

use super::book::{Adts, Ctrs};
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
    Any(Py<PyAny>),
}

pub(crate) trait BendCtr: std::fmt::Debug {
    fn to_py(&self, py: &Python) -> Py<PyAny>;
}

pub fn from_term_into_adt(term: &BTerm, def_adts: &Ctrs) -> Option<TermParse> {
    match term {
        BTerm::Lam { tag, pat, bod } => {
            if let BTerm::App { tag, fun, arg } = bod.as_ref() {
                let arg_adt = from_term_into_adt(arg.as_ref(), def_adts);

                if let BTerm::App { tag, fun, arg } = fun.as_ref() {
                    let mut n: i32 = 0;

                    let inside_arg = from_term_into_adt(arg.as_ref(), def_adts);

                    if let BTerm::App { tag, fun, arg } = fun.as_ref() {
                        if let BTerm::Num { val } = arg.as_ref() {
                            n = num_to_i32(val);

                            match n {
                                1 => {
                                    let mut adt =
                                        def_adts.second.clone().unwrap();

                                    let mut py_obj: Option<TermParse> = None;

                                    let mut elements: Vec<Py<PyAny>> = vec![];

                                    Python::with_gil(|py| {
                                        if let Some(inside_arg) = inside_arg {
                                            match inside_arg {
                                                TermParse::I32(val) => elements
                                                    .push(val.into_py(py)),
                                                TermParse::Any(py_any) => {
                                                    elements.push(py_any)
                                                }
                                            }
                                        }

                                        if let Some(inside_arg) = arg_adt {
                                            match inside_arg {
                                                TermParse::I32(val) => elements
                                                    .push(val.into_py(py)),
                                                TermParse::Any(py_any) => {
                                                    elements.push(py_any)
                                                }
                                            }
                                        }

                                        if elements.len() == 1 {
                                            let adt =
                                                def_adts.first.clone().unwrap();
                                            elements.push(adt.to_py(&py));
                                        }

                                        let args =
                                            PyTuple::new_bound(py, elements);

                                        py_obj = Some(TermParse::Any(
                                            adt.__call__(args).unwrap(),
                                        ));
                                    });

                                    return py_obj;
                                }
                                _ => {
                                    dbg!("should treat the last numarg");
                                    return None;
                                }
                            }
                        }
                    }
                }
            }
            None
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
