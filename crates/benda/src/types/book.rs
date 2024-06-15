use bend::fun::{
    Adt as BendAdt, Book as BendBook, CtrField, Definition as BendDef,
};
use indexmap::IndexMap;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::types::{PyNone, PyString};

use super::BuiltinType;

fn new_err<T>(str: String) -> PyResult<T> {
    Err(PyException::new_err(str))
}

#[pyclass(name = "Ctr")]
#[derive(Clone, Debug, Default)]
pub struct Ctr {
    fields: IndexMap<String, Option<Py<PyAny>>>,
}

#[pymethods]
impl Ctr {
    fn __setattr__(&mut self, field: Bound<PyAny>, value: Bound<PyAny>) {
        println!("FIELD {:?}", field.to_string());

        if let Some(val) = self.fields.get_mut(&field.to_string()) {
            val.replace(value.to_object(field.py()));
        }
    }

    fn __getattr__(&self, object: Bound<PyAny>) -> PyResult<PyObject> {
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
    fields: IndexMap<String, Ctr>,
}

impl Ctrs {
    fn new() -> Self {
        Self {
            fields: IndexMap::new(),
        }
    }
}

#[pymethods]
impl Ctrs {
    fn __getattr__(&self, object: Bound<PyAny>) -> PyResult<Py<Ctr>> {
        let py = object.py();

        if let Some(val) = self.fields.get(&object.to_string()) {
            Ok(Python::with_gil(|py| Py::new(py, val.clone()).unwrap()))
        } else {
            new_err(format!("Could not find attr {}", object))
        }
    }
}

#[pyclass(name = "Definitions")]
#[derive(Clone, Debug, Default)]
pub struct Definitions {}

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
    bend_book: BendBook,
    adts: Adts,
    defs: Definitions,
}

impl Book {
    pub fn new(bend_book: BendBook) -> Self {
        let mut adts = Adts::new();

        for (adt_name, bend_adt) in bend_book.adts.iter() {
            let mut new_adt = Adts::new();

            let mut all_ctrs = Ctrs::default();

            for (ctr_name, ctr_fields) in bend_adt.ctrs.iter() {
                let new_name = ctr_name.split('/').last().unwrap().to_string();
                let mut new_ctr = Ctr::default();

                for c in ctr_fields {
                    new_ctr.fields.insert(c.nam.to_string(), None);
                }

                all_ctrs.fields.insert(new_name, new_ctr);
            }

            adts.adts.insert(adt_name.to_string(), all_ctrs);
        }

        println!("\n\nBend: {:?}\n", bend_book.adts);
        println!("\n\nBenda: {:?}\n", adts);

        Self {
            bend_book,
            adts,
            defs: Definitions::default(),
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
                //let adt = self.defs.get(&field.to_string()).unwrap();
                //Ok(adt.clone().into_py(py))
                new_err("Not yet Implemented".to_string())
            }

            _ => new_err(format!("Could not find attribute {}", object)),
        }
    }
}
