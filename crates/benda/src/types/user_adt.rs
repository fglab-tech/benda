use bend::fun::Adt;
use pyo3::type_object::HasPyGilRef;
use pyo3::types::PyAnyMethods;
use pyo3::{FromPyObject, PyTypeCheck};

use super::BendType;

struct UserAdt {
    adt: Adt,
}

impl<'py> FromPyObject<'py> for UserAdt {
    fn extract(ob: &'py pyo3::PyAny) -> pyo3::PyResult<Self> {
        for field in ob.iter() {
            println!("{:?}", field);
        }

        //Self::extract_bound(&ob.as_borrowed())
    }

    fn extract_bound(
        ob: &pyo3::Bound<'py, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        Self::extract(ob.clone().into_gil_ref())
    }
}

impl BendType for UserAdt {
    fn to_bend(&self) -> super::ToBendResult {
        println!("ADT: {:?}", self.adt);
        todo!()
    }
}
