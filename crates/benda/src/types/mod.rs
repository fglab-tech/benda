use core::panic;

use bend::{fun::Num, imp};

use pyo3::{
    types::{PyAnyMethods, PyFloat, PyTypeMethods},
    Bound, FromPyObject, PyAny, PyTypeCheck, ToPyObject,
};
use tree::{Leaf, Node, Tree};

pub mod f24;
pub mod i24;
pub mod tree;
pub mod u24;

pub trait BendType {
    fn to_bend(&self) -> imp::Expr;
}

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

pub fn extract_num(arg: Bound<PyAny>, t_type: BuiltinType) -> Option<imp::Expr> {
    match t_type {
        BuiltinType::I32 => Some(arg.to_string().parse::<i32>().unwrap().to_bend()),
        BuiltinType::F32 => Some(arg.to_string().parse::<f32>().unwrap().to_bend()),
        _ => unreachable!(),
    }
}

pub fn extract_type(arg: Bound<PyAny>) -> Option<imp::Expr> {
    let t_type = arg.get_type();
    let name = t_type.name().unwrap();

    let arg_type = BuiltinType::from(name.to_string());

    match arg_type {
        BuiltinType::U24 => Some(extract_inner::<crate::u24>(arg).unwrap().to_bend()),
        BuiltinType::I32 => extract_num(arg, BuiltinType::I32),
        BuiltinType::F32 => extract_num(arg, BuiltinType::F32),
        BuiltinType::Tree => Some(extract_inner::<Tree>(arg).unwrap().to_bend()),
        BuiltinType::Node => Some(extract_inner::<Node>(arg).unwrap().to_bend()),
        BuiltinType::Leaf => Some(extract_inner::<Leaf>(arg).unwrap().to_bend()),
        _ => panic!(),
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
}

impl From<String> for BuiltinType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "float" => BuiltinType::F32,
            "int" => BuiltinType::I32,
            "benda.u24" => BuiltinType::U24,
            "benda.Node" => BuiltinType::Node,
            "benda.Leaf" => BuiltinType::Node,
            "benda.Tree" => BuiltinType::Tree,
            _ => panic!("Could not parse type"),
        }
    }
}

impl BendType for u32 {
    fn to_bend(&self) -> imp::Expr {
        imp::Expr::Num {
            val: Num::U24(*self),
        }
    }
}

impl BendType for f32 {
    fn to_bend(&self) -> imp::Expr {
        imp::Expr::Num {
            val: Num::F24(*self),
        }
    }
}

impl BendType for i32 {
    fn to_bend(&self) -> imp::Expr {
        imp::Expr::Num {
            val: Num::I24(*self),
        }
    }
}

impl BendType for PyFloat {
    fn to_bend(&self) -> imp::Expr {
        let num: f32 = self.extract().unwrap();
        imp::Expr::Num { val: Num::F24(num) }
    }
}
