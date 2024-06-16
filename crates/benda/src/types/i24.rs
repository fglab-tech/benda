use std::ops::{Add, Sub};

use bend::imp;
use pyo3::{pyclass, pymethods};

use super::{BendType, ToBendResult};

#[pyclass(module = "benda")]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct I24(i32);

impl BendType for I24 {
    fn to_bend(&self) -> ToBendResult {
        Ok(imp::Expr::Num {
            val: bend::fun::Num::I24(self.0),
        })
    }
}

impl I24 {
    const MAX: i32 = 0xffffff;

    // TODO: Check if the masking is working properly
    pub fn new(value: i32) -> Self {
        Self(value & Self::MAX)
    }

    pub fn get(self) -> i32 {
        self.0
    }
}

impl std::fmt::Debug for I24 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for I24 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<i32> for I24 {
    fn from(value: i32) -> Self {
        I24::new(value)
    }
}

impl From<I24> for i32 {
    fn from(val: I24) -> Self {
        val.0
    }
}

impl std::ops::Add for I24 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        I24::new(self.0 + other.0)
    }
}

impl std::ops::Sub for I24 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        I24::new(self.0 - other.0)
    }
}

// TODO: Check for overflow on the operations
// TODO: Implement tests for each operation comparing to Bend

#[pymethods]
impl I24 {
    #[new]
    fn new_py(value: i32) -> Self {
        I24::new(value)
    }

    fn __add__(&self, other: &Self) -> Self {
        I24::add(*self, *other)
    }

    fn __sub__(&self, other: &Self) -> Self {
        I24::sub(*self, *other)
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }
}
