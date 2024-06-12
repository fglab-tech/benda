use std::ops::{Add, Sub};

use bend::imp;
use pyo3::{pyclass, pymethods};

use super::{BendType, ToBendResult};

#[pyclass(module = "benda")]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct f24(f32);

impl BendType for f24 {
    fn to_bend(&self) -> ToBendResult {
        Ok(imp::Expr::Num {
            val: bend::fun::Num::F24(self.0),
        })
    }
}

impl f24 {
    // TODO: Implement a masking for float numbers.
    pub fn new(value: f32) -> Self {
        Self(value)
    }

    pub fn get(self) -> f32 {
        self.0
    }
}

impl std::fmt::Debug for f24 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for f24 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<f32> for f24 {
    fn from(value: f32) -> Self {
        f24::new(value)
    }
}

impl From<f24> for f32 {
    fn from(val: f24) -> Self {
        val.0
    }
}

impl std::ops::Add for f24 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        f24::new(self.0 + other.0)
    }
}

impl std::ops::Sub for f24 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        f24::new(self.0 - other.0)
    }
}

// TODO: Check for overflow on the operations
// TODO: Implement tests for each operation comparing to Bend

#[pymethods]
impl f24 {
    #[new]
    fn new_py(value: f32) -> Self {
        f24::new(value)
    }

    fn __add__(&self, other: &Self) -> Self {
        f24::add(*self, *other)
    }

    fn __sub__(&self, other: &Self) -> Self {
        f24::sub(*self, *other)
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }
}
