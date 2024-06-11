use std::ops::{Add, Sub};

use bend::imp;
use pyo3::{pyclass, pymethods};

use super::BendType;

#[pyclass(module = "benda")]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct i24(i32);

impl BendType for i24 {
    fn to_bend(&self) -> bend::imp::Expr {
        imp::Expr::Num {
            val: bend::fun::Num::I24(self.0),
        }
    }
}

impl i24 {
    const MAX: i32 = 0xffffff;

    pub fn new(value: i32) -> Self {
        Self(value & Self::MAX)
    }

    pub fn get(self) -> i32 {
        self.0
    }
}

impl std::fmt::Debug for i24 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for i24 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<i32> for i24 {
    fn from(value: i32) -> Self {
        i24::new(value)
    }
}

impl From<i24> for i32 {
    fn from(val: i24) -> Self {
        val.0
    }
}

impl std::ops::Add for i24 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        i24::new(self.0 + other.0)
    }
}

impl std::ops::Sub for i24 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        i24::new(self.0 - other.0)
    }
}

#[pymethods]
impl i24 {
    #[new]
    fn new_py(value: i32) -> Self {
        i24::new(value)
    }

    fn __add__(&self, other: &Self) -> Self {
        i24::add(*self, *other)
    }

    fn __sub__(&self, other: &Self) -> Self {
        i24::sub(*self, *other)
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }
}
