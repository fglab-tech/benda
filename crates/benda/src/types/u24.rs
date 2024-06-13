use std::ops::{Add, Sub};

use bend::imp;
use pyo3::{pyclass, pymethods};

use super::{BendType, ToBendResult};

#[pyclass(module = "benda")]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct U24(u32);

impl BendType for U24 {
    fn to_bend(&self) -> ToBendResult {
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

    fn __str__(&self) -> String {
        self.0.to_string()
    }
}
