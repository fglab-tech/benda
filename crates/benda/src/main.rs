
use pyo3::{prelude::*, types::{PyCode, PyFunction}};

use python_ast::parse;

mod parser;
mod bend;

fn main() -> PyResult<()> {

    pyo3::prepare_freethreaded_python();

    let code = std::fs::read_to_string("main.py").unwrap();

    let ast = parse(&code, "main.py").unwrap();

    println!("AST : {:?}", ast);

    Python::with_gil(|py| {
        let fun = PyModule::from_code_bound(
            py,
            &code,
            "main.py",
            "example"
        ).unwrap()
        .getattr("print_ast")
        .unwrap();
    

        let module = fun.downcast::<PyFunction>();

        match module {
            Ok(m) => {
                match m.downcast::<PyCode>() {
                    Ok(c) => {
                        println!("{:?}", c);

                    },
                    Err(_) => todo!(),
                };
            },
            _ => panic!("O"),
        };


        //println!("AST gerada pelo Python: \n{}\n\n", ast);

        //let mut scanner = Scanner::new(ast);

        //scanner.scan_tokens();
        //let tokens = scanner.tokens;

        //let mut parser = Parser::new(tokens);
        //parser.parse();

    });

    //parser::run::run();

    Ok(())
}