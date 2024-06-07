mod parser;

use parser::Parser;
use pyo3::prelude::*;

use rustpython_parser::{ parse, Mode };

mod benda_ffi;

fn main() -> PyResult<()> {
    let filename = String::from("main.py");

    let code = std::fs::read_to_string(&filename).unwrap();
    let module = parse(code.as_str(), Mode::Module, "main.py").unwrap();

    match module {
        rustpython_parser::ast::Mod::Module(mods) => {
            let mut parser = Parser::new(mods.body, 0);
            parser.parse(&String::from("sum_tree"), &["tree".to_string()]);
        }
        _ => todo!(),
    }

    Ok(())
}
