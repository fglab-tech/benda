use bend::fun::{Adt, Book, Name};
use bend::imp::{self};
use pyo3::types::{PyAnyMethods, PyString, PyTypeMethods};
use pyo3::{Bound, PyAny, PyErr};

use super::{extract_type_raw, BendType};

pub struct UserAdt<'py> {
    adt: Adt,
    entire_nam: Name,
    data: Bound<'py, PyAny>,
    book: Book,
}

impl<'py> UserAdt<'py> {
    pub fn new(data: Bound<'py, PyAny>, book: &Book) -> Option<Self> {
        let t_type = data.get_type();

        let binding = t_type.name().unwrap();

        for (nam, _ctr) in &book.ctrs {
            let new_nam = nam.to_string();
            let two_names = new_nam.split_once('/').unwrap();

            if two_names.1 == binding {
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
                        return Ok(imp::Expr::Ctr {
                            name: nam.clone(),
                            args: vec![t.to_bend().unwrap()],
                            kwargs: vec![],
                        });
                    }

                    if let Some(adt) = UserAdt::new(attr, &self.book) {
                        let new_adt = adt.to_bend();
                        adt_fields.push(new_adt.unwrap());
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
