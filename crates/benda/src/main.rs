mod parser;

use pyo3::prelude::*;
use rustpython_parser::{parse, Mode};

mod benda_ffi;

fn main() -> PyResult<()> {
    let filename = String::from("main.py");

    let code = std::fs::read_to_string(filename).unwrap();
    let module = parse(code.as_str(), Mode::Module, "main.py").unwrap();

    match module {
        rustpython_parser::ast::Mod::Module(_mods) => {
            //let mut parser = Parser::new(mods.body, 0);
            //let val = parser.parse(&String::from("sum_tree"), &["tree".to_string()]);
            //println!("Return {:?}", val);
        }
        _ => todo!(),
    }

    Ok(())
}
