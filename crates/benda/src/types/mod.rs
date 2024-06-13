use bend::fun::{Book, Num};
use bend::imp::{self};
use pyo3::types::{PyAnyMethods, PyFloat, PyTypeMethods};
use pyo3::{Bound, FromPyObject, PyAny, PyErr, PyTypeCheck};
use tree::{Leaf, Node, Tree};
use user_adt::UserAdt;

pub mod f24;
pub mod i24;
pub mod tree;
pub mod u24;
pub mod user_adt;

pub trait BendType {
    fn to_bend(&self) -> ToBendResult;
}

type ToBendResult = Result<imp::Expr, PyErr>;

pub fn extract_inner<'py, T: BendType + PyTypeCheck + FromPyObject<'py>>(
    arg: Bound<'py, PyAny>,
) -> Option<T> {
    let inner = arg.downcast::<T>();
    if let Ok(inner) = inner {
        let inner = <T as FromPyObject>::extract_bound(inner.as_any());
        return Some(inner.unwrap());
    }
    None
}

pub fn extract_num_raw(
    arg: Bound<PyAny>,
    t_type: BuiltinType,
) -> Box<dyn BendType> {
    match t_type {
        BuiltinType::I32 => Box::new(arg.to_string().parse::<i32>().unwrap()),
        BuiltinType::F32 => Box::new(arg.to_string().parse::<f32>().unwrap()),
        _ => unreachable!(),
    }
}

pub fn extract_num(arg: Bound<PyAny>, t_type: BuiltinType) -> ToBendResult {
    match t_type {
        BuiltinType::I32 => arg.to_string().parse::<i32>().unwrap().to_bend(),
        BuiltinType::F32 => arg.to_string().parse::<f32>().unwrap().to_bend(),
        _ => unreachable!(),
    }
}

pub fn extract_type_raw(arg: Bound<PyAny>) -> Option<Box<dyn BendType>> {
    let t_type = arg.get_type();
    let name = t_type.name().unwrap();

    let arg_type = BuiltinType::from(name.to_string());

    match arg_type {
        BuiltinType::U24 => {
            Some(Box::new(extract_inner::<u24::u24>(arg).unwrap()))
        }
        BuiltinType::I32 => Some(extract_num_raw(arg, BuiltinType::I32)),
        BuiltinType::F32 => Some(extract_num_raw(arg, BuiltinType::F32)),
        _ => None,
    }
}

pub fn extract_type(arg: Bound<PyAny>, book: &Book) -> ToBendResult {
    let t_type = arg.get_type();
    let name = t_type.name().unwrap();

    let arg_type = BuiltinType::from(name.to_string());

    match arg_type {
        BuiltinType::U24 => extract_inner::<u24::u24>(arg).unwrap().to_bend(),
        BuiltinType::I32 => extract_num(arg, BuiltinType::I32),
        BuiltinType::F32 => extract_num(arg, BuiltinType::F32),
        BuiltinType::Tree => extract_inner::<Tree>(arg).unwrap().to_bend(),
        BuiltinType::Node => extract_inner::<Node>(arg).unwrap().to_bend(),
        BuiltinType::Leaf => extract_inner::<Leaf>(arg).unwrap().to_bend(),
        BuiltinType::UserAdt => UserAdt::new(arg, book).unwrap().to_bend(),
    }
}

#[derive(Debug)]
pub enum BuiltinType {
    U24,
    F32,
    I32,
    Tree,
    Leaf,
    Node,
    UserAdt,
}

impl From<String> for BuiltinType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "float" => BuiltinType::F32,
            "int" => BuiltinType::I32,
            "benda.u24" => BuiltinType::U24,
            "benda.Node" => BuiltinType::Node,
            "benda.Leaf" => BuiltinType::Leaf,
            "benda.Tree" => BuiltinType::Tree,
            _ => BuiltinType::UserAdt,
        }
    }
}

impl BendType for u32 {
    fn to_bend(&self) -> ToBendResult {
        Ok(imp::Expr::Num {
            val: Num::U24(*self),
        })
    }
}

impl BendType for f32 {
    fn to_bend(&self) -> ToBendResult {
        Ok(imp::Expr::Num {
            val: Num::F24(*self),
        })
    }
}

impl BendType for i32 {
    fn to_bend(&self) -> ToBendResult {
        Ok(imp::Expr::Num {
            val: Num::I24(*self),
        })
    }
}

impl BendType for PyFloat {
    fn to_bend(&self) -> ToBendResult {
        let num: Result<f32, PyErr> = self.extract();

        match num {
            Ok(num) => Ok(imp::Expr::Num { val: Num::F24(num) }),
            Err(err) => Err(err),
        }
    }
}
