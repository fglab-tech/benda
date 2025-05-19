use std::ops::{Add, Sub};

use bend::imp;
use num_traits::ToPrimitive;
use pyo3::basic::CompareOp;
use pyo3::exceptions::PyZeroDivisionError;
use pyo3::{pyclass, pymethods, PyResult};

use super::{BendResult, BendType};

#[pyclass(module = "benda", name = "u24")]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct U24(u32);

impl BendType for U24 {
    fn to_bend(&self) -> BendResult {
        Ok(imp::Expr::Num {
            val: bend::fun::Num::U24(self.0),
        })
    }
}

impl U24 {
    const MAX: u32 = 0xffffff;

    // TODO: Check if the masking is working properly
    pub fn new(value: u32) -> Self {
        Self(value & Self::MAX)
    }

    pub fn get(self) -> u32 {
        self.0
    }
}

impl std::fmt::Debug for U24 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for U24 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u32> for U24 {
    fn from(value: u32) -> Self {
        U24::new(value)
    }
}

impl From<U24> for u32 {
    fn from(val: U24) -> Self {
        val.0
    }
}

impl std::ops::Add for U24 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        U24::new(self.0 + other.0)
    }
}

impl std::ops::Sub for U24 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        U24::new(self.0 - other.0)
    }
}

// TODO: Check for overflow on the operations
// TODO: Implement tests for each operation comparing to Bend

#[pymethods]
impl U24 {
    #[new]
    fn new_py(value: u32) -> Self {
        U24::new(value)
    }

    fn __add__(&self, other: &Self) -> Self {
        U24::add(*self, *other)
    }

    fn __sub__(&self, other: &Self) -> Self {
        U24::sub(*self, *other)
    }

    fn __mul__(&self, other: &Self) -> Self {
        Self(self.0.wrapping_mul(other.0))
    }

    fn __truediv__(&self, other: &Self) -> PyResult<Self> {
        match self.0.checked_div(other.0) {
            Some(i) => Ok(Self(i)),
            None => Err(PyZeroDivisionError::new_err("division by zero")),
        }
    }

    fn __floordiv__(&self, other: &Self) -> PyResult<Self> {
        match self.0.checked_div(other.0) {
            Some(i) => Ok(Self(i)),
            None => Err(PyZeroDivisionError::new_err("division by zero")),
        }
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }

    fn __repr__(&self) -> String {
        self.0.to_string()
    }

    fn __int__(&self) -> i32 {
        self.0.to_i32().unwrap()
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        match op {
            pyclass::CompareOp::Lt => Ok(self < other),
            pyclass::CompareOp::Le => Ok(self <= other),
            pyclass::CompareOp::Eq => Ok(self == other),
            pyclass::CompareOp::Ne => Ok(self != other),
            pyclass::CompareOp::Gt => Ok(self > other),
            pyclass::CompareOp::Ge => Ok(self >= other),
        }
    }
}

#[cfg(test)]
mod u24_tests {
    use core::panic;
    use std::fmt::format;
    use std::fs::File;
    use std::path::Path;

    use bend::fun::Term;
    use bend::run_book;
    use indexmap::IndexMap;

    use super::*;
    use crate::benda_ffi;
    use crate::types::book::{BendRuntime, Ctrs};
    use crate::types::user_adt::{from_term_into_adt, TermParse};

    fn run_bend_code(code: &str) -> Term {
        let code = format!("(Main) = ({})", code);

        let path = Path::new("bend.tmp");
        let _ = File::create_new(path);

        let book = bend::fun::load_book::do_parse_book(
            code.as_str(),
            path,
            bend::fun::Book::builtins(),
        );

        if let Ok(res) = benda_ffi::run(
            &book.unwrap(),
            BendRuntime::Rust.to_string().as_str(),
        ) {
            match res {
                Some((ter, _, _)) => return ter,
                None => {
                    panic!("Could not get result from HVM")
                }
            }
        }
        panic!("Could not get result from HVM")
    }

    fn extract_i32(term: &Term) -> i32 {
        let res = from_term_into_adt(term, &Ctrs::default());

        match res {
            Some(TermParse::I32(val)) => val,
            _ => {
                todo!()
            }
        }
    }

    #[test]
    fn overflow() {
        let bend_res = run_bend_code("(<< 1 25)");
        let benda_res = U24::new(1 << 25);

        let bend_res = extract_i32(&bend_res);

        assert_eq!(bend_res.to_u32().unwrap(), benda_res.0);
    }

    #[test]
    fn sum() {
        let bend_res = run_bend_code("(+ 25 10)");
        let benda_res = U24::new(25 + 10);

        let bend_res = extract_i32(&bend_res);

        assert_eq!(bend_res.to_u32().unwrap(), benda_res.0);
    }
}
