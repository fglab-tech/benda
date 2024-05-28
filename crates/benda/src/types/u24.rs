use std::ops::{Add, Sub};

use pyo3::{pyclass, pymethods};

#[pyclass(module = "benda_py")]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct u24(u32);

impl u24 {
    const MAX: u32 = 0xFFFFFF;

    pub fn new(value: u32) -> Self {
        Self(value & Self::MAX)
    }

    pub fn get(self) -> u32 {
        self.0
    }

}

impl std::fmt::Debug for u24 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for u24 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u32> for u24 {
    fn from(value: u32) -> Self {
        u24::new(value)
    }
}

impl Into<u32> for u24 {
    fn into(self) -> u32 {
        self.0
    }
}

impl std::ops::Add for u24 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        u24::new(self.0 + other.0)
    }
}

impl std::ops::Sub for u24 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        u24::new(self.0 - other.0)
    }
}


#[pymethods]
impl u24 {


    #[new]
    fn new_py(value: u32) -> Self {
        u24::new(value)
    }

    fn __add__(&self, other: &Self) -> Self {
        u24::add(*self, *other)
    }

    fn __sub__(&self, other: &Self) -> Self {
        u24::sub(*self, *other)
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }
}
