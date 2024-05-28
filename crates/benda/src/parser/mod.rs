use core::panic;

use bend::fun::{ Book, Definition, Name, Pattern, Rule, Term };
use python_ast::Statement;

use crate::bend::run;

pub struct Parser {
    statements: Vec<Statement>,
}

impl Parser {
    pub fn parse(statements: Vec<Statement>) {
        let mut book = Book::default();

        let mut fun_body: Vec<Rule> = vec![];

        for stmt in statements {
            match stmt.statement {
                python_ast::StatementType::Assign(assign) => {
                    // TODO: Implement tuple assignment
                    let target = &assign.targets.get(0).unwrap().id;
                    let value: u32 = match assign.value {
                        python_ast::ExprType::Constant(c) => c.to_string().parse().unwrap(),
                        _ => { panic!("Could not get assignment value.") }
                    };

                    let rule = Rule {
                        pats: vec![Pattern::Var(Some(Name::new(target)))],
                        body: Term::Nat { val: value },
                    };

                    fun_body.push(rule);
                }
                python_ast::StatementType::Call(call) => {
                    let expr_type = *call.func;

                    match expr_type {
                        python_ast::ExprType::BoolOp(_) => {},
                        python_ast::ExprType::NamedExpr(_) => todo!(),
                        python_ast::ExprType::BinOp(_) => todo!(),
                        python_ast::ExprType::UnaryOp(_) => todo!(),
                        python_ast::ExprType::Await(_) => todo!(),
                        python_ast::ExprType::Compare(_) => todo!(),
                        python_ast::ExprType::Call(_) => todo!(),
                        python_ast::ExprType::Constant(_) => todo!(),
                        python_ast::ExprType::Attribute(_) => todo!(),
                        python_ast::ExprType::Name(_) => todo!(),
                        python_ast::ExprType::List(_) => todo!(),
                        python_ast::ExprType::NoneType(_) => todo!(),
                        python_ast::ExprType::Unimplemented(_) => todo!(),
                        python_ast::ExprType::Unknown => todo!(),
                    }
                }
                _ => {}
            };
        }

        book.defs.insert(Name::new("main"), Definition {
            name: Name::new("main"),
            rules: fun_body,
            builtin: false,
        });
        println!("BEND:\n {}", book.display_pretty());
        run(&book);
    }
}
