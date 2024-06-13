use core::panic;
use std::vec;

use bend::{fun, imp};
use pyo3::types::{PyAnyMethods, PyTuple, PyTypeMethods};
use pyo3::{pyclass, pymethods, Bound};

use super::u24::U24;
use super::{BendType, ToBendResult};
use crate::types::extract_inner;

#[derive(Clone, Debug)]
#[pyclass(module = "benda", name = "Leaf")]
pub struct Leaf {
    pub value: U24,
}

impl BendType for Leaf {
    fn to_bend(&self) -> ToBendResult {
        self.value.to_bend()
    }
}

#[pymethods]
impl Leaf {
    #[new]
    fn __new__(val: u32) -> Self {
        Self {
            value: U24::new(val),
        }
    }
}

#[derive(Clone, Debug)]
#[pyclass(module = "benda", name = "Node")]
pub struct Node {
    pub left: Option<Box<Tree>>,
    pub right: Option<Box<Tree>>,
}

#[pymethods]
impl Node {
    #[new]
    #[pyo3(signature = (*py_args))]
    fn new(py_args: &Bound<'_, PyTuple>) -> Self {
        let mut trees: Option<Tree> = None;

        for arg in py_args {
            let t_type = arg.get_type();
            let name = t_type.name().unwrap();

            let tree_type = TreeType::from(name.to_string());

            let new_tree: Option<Tree> = match tree_type {
                TreeType::Leaf => extract_inner::<Leaf>(arg).map(|leaf| Tree {
                    leaf: Some(leaf),
                    node: None,
                }),
                TreeType::Node => extract_inner::<Node>(arg).map(|node| Tree {
                    leaf: None,
                    node: Some(node),
                }),
                TreeType::Tree => extract_inner::<Tree>(arg),
            };

            if let Some(new_tree) = new_tree {
                if let Some(tree) = trees {
                    return Self {
                        left: Some(Box::new(tree)),
                        right: Some(Box::new(new_tree)),
                    };
                } else {
                    trees = Some(new_tree);
                }
            }
        }

        panic!("Node must receive two trees in its constructor")
    }
}

impl BendType for Node {
    fn to_bend(&self) -> ToBendResult {
        let mut trees: Vec<imp::Expr> = vec![];

        if let Some(left) = &self.left {
            match left.to_bend() {
                Ok(val) => trees.push(val),
                Err(err) => return Err(err),
            };
        }

        if let Some(right) = &self.right {
            match right.to_bend() {
                Ok(val) => trees.push(val),
                Err(err) => return Err(err),
            };
        }

        Ok(imp::Expr::Ctr {
            name: fun::Name::new("Tree/Node"),
            args: trees,
            kwargs: vec![],
        })
    }
}

#[derive(Clone, Debug)]
#[pyclass(module = "benda", name = "Tree")]
pub struct Tree {
    pub leaf: Option<Leaf>,
    pub node: Option<Node>,
}

impl BendType for Tree {
    fn to_bend(&self) -> ToBendResult {
        if let Some(leaf) = &self.leaf {
            return leaf.to_bend();
        }

        if let Some(node) = &self.node {
            return node.to_bend();
        }

        todo!()
    }
}

#[derive(Debug)]
pub enum TreeType {
    Leaf,
    Node,
    Tree,
}

impl From<String> for TreeType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "benda.Leaf" => TreeType::Leaf,
            "benda.Node" => TreeType::Node,
            "benda.Tree" => TreeType::Tree,
            _ => panic!("Tree __new__ must receive either Leaf or Node"),
        }
    }
}

#[pymethods]
impl Tree {
    #[new]
    #[pyo3(signature = (*py_args))]
    fn new(py_args: &Bound<'_, PyTuple>) -> Self {
        for arg in py_args {
            let t_type = arg.get_type();
            let name = t_type.name().unwrap();
            let tree_type = TreeType::from(name.to_string());

            match tree_type {
                TreeType::Leaf => {
                    let leaf: Option<Leaf> = extract_inner(arg);
                    if let Some(leaf) = leaf {
                        return Self {
                            leaf: Some(leaf),
                            node: None,
                        };
                    }
                }
                _ => {
                    panic!("Tree must receive a Leaf in constructor");
                }
            }
        }

        todo!()
    }
}
