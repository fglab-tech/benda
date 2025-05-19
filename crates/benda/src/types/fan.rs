use bend::fun::Term;
use pyo3::types::{PyTuple, PyTupleMethods};
use pyo3::{pyclass, pymethods, Bound};

use crate::types::extract_type_raw;

#[pyclass(name = "Fan")]
#[derive(Clone, Debug, Default)]
pub struct Fan {
    pub term: Term,
}

#[pymethods]
impl Fan {
    #[new]
    #[pyo3(signature = (*args))]
    fn new(args: Bound<'_, PyTuple>) -> Self {
        let mut elements: Vec<Term> = vec![];
        for arg in args.iter() {
            let new_arg =
                extract_type_raw(arg.clone()).unwrap().to_bend().unwrap();
            let u_type = Some(new_arg.clone().to_fun());

            if let Some(u_type) = u_type {
                elements.push(u_type);
            }
        }

        let fan = bend::fun::Term::Fan {
            fan: bend::fun::FanKind::Tup,
            tag: bend::fun::Tag::Static,
            els: elements,
        };

        Self { term: fan }
    }
}
