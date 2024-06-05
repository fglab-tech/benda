use core::panic;
use std::vec;

use bend::fun::{Adt, Book, CtrField, Name, Op};
use bend::imp::{self, Expr, MatchArm, Stmt};
use indexmap::IndexMap;
use num_traits::cast::ToPrimitive;
use rustpython_parser::ast::{
    located, CmpOp as rCmpOp, Expr as rExpr, ExprBinOp, Operator as rOperator,
    Pattern as rPattern, Stmt as rStmt, StmtAssign, StmtMatch,
};
use rustpython_parser::text_size::TextRange;

use crate::benda_ffi::run;

#[derive(Clone, Debug)]
enum FromExpr {
    CtrField(Vec<CtrField>),
    Expr(imp::Expr),
    Statement(imp::Stmt),
}

pub struct Parser {
    statements: Vec<rStmt<TextRange>>,
    book: Book,
    definitions: Vec<imp::Definition>,
}

impl Parser {
    pub fn new(statements: Vec<rStmt<TextRange>>, _index: usize) -> Self {
        Self {
            statements,
            //book:  fun::Book::builtins(),
            book: Book::default(),
            definitions: vec![],
        }
    }

    fn parse_expr_type_with_sub(
        &self,
        expr: Box<rExpr<TextRange>>,
        vars: &Vec<String>,
        subs: &Vec<String>,
    ) -> Option<FromExpr> {
        match *expr {
            rExpr::Name(n) => {
                let mut name: Option<String> = None;

                for var in vars.iter() {
                    if n.id == *var {
                        name =
                            Some(format!("{}.{}", subs.first().unwrap(), var));
                    }
                }

                if name.is_none() {
                    name = Some(n.id.to_string());
                }

                Some(FromExpr::Expr(imp::Expr::Var {
                    nam: Name::new(name.unwrap()),
                }))
            }
            rExpr::BinOp(bin_op) => {
                self.parse_bin_op(bin_op, Some(vars), Some(subs))
            }
            rExpr::Call(c) => {
                let fun = c.func;

                let expr = self.parse_expr_type(fun);

                if let Some(FromExpr::Expr(Expr::Var { ref nam })) = expr {
                    let mut args: Vec<Expr> = vec![];

                    for arg in c.args {
                        let arg = self.parse_expr_type_with_sub(
                            Box::new(arg),
                            vars,
                            subs,
                        );

                        if let Some(FromExpr::Expr(e)) = arg {
                            args.push(e);
                        }
                    }
                    if let Some(val) = self.book.adts.get(&nam.clone()) {
                        return Some(FromExpr::Expr(imp::Expr::Constructor {
                            name: val.ctrs.first().unwrap().0.clone(),
                            args,
                            kwargs: vec![],
                        }));
                    }
                    return Some(FromExpr::Expr(imp::Expr::Call {
                        fun: Box::new(Expr::Var {
                            nam: Name::new(nam.to_string()),
                        }),
                        args,
                        kwargs: vec![],
                    }));
                }
                expr
            }
            _ => self.parse_expr_type(expr),
        }
    }

    #[allow(clippy::boxed_local)]
    fn parse_expr_type(&self, expr: Box<rExpr<TextRange>>) -> Option<FromExpr> {
        match *expr {
            rExpr::Attribute(att) => {
                if let FromExpr::Expr(imp::Expr::Var { nam: lib }) =
                    self.parse_expr_type(att.value).unwrap()
                {
                    let fun = att.attr.to_string();
                    #[allow(clippy::cmp_owned)]
                    if lib.to_string() == "benda" && fun == "switch" {
                        return Some(FromExpr::Expr(Expr::Call {
                            fun: Box::new(Expr::Var {
                                nam: Name::new("switch"),
                            }),
                            args: vec![],
                            kwargs: vec![],
                        }));
                    }
                }
                None
            }
            rExpr::Compare(comp) => {
                let left = self.parse_expr_type(comp.left);
                let right = self.parse_expr_type(Box::new(
                    comp.comparators.first().unwrap().clone(),
                ));

                let op = match comp.ops.first().unwrap() {
                    rCmpOp::Eq => Op::EQ,
                    rCmpOp::NotEq => Op::NEQ,
                    rCmpOp::Lt => Op::LT,
                    rCmpOp::LtE => todo!(),
                    rCmpOp::Gt => Op::GT,
                    rCmpOp::GtE => todo!(),
                    rCmpOp::Is => todo!(),
                    rCmpOp::IsNot => todo!(),
                    rCmpOp::In => todo!(),
                    rCmpOp::NotIn => todo!(),
                };

                if let (FromExpr::Expr(left), FromExpr::Expr(right)) =
                    (left.unwrap(), right.unwrap())
                {
                    return Some(FromExpr::Expr(Expr::Bin {
                        op,
                        lhs: Box::new(left),
                        rhs: Box::new(right),
                    }));
                }
                None
            }
            rExpr::BinOp(bin_op) => self.parse_bin_op(bin_op, None, None),
            rExpr::Constant(c) => match c.value {
                located::Constant::None => todo!(),
                located::Constant::Bool(_) => todo!(),
                located::Constant::Str(str) => {
                    let nam = Name::new(str);
                    let adt = self.book.adts.get(&nam);

                    if let Some(_adt) = adt {
                        return Some(FromExpr::Expr(imp::Expr::Var { nam }));
                    }
                    None
                }
                located::Constant::Bytes(_) => todo!(),
                located::Constant::Int(val) => {
                    Some(FromExpr::Expr(imp::Expr::Num {
                        val: bend::fun::Num::U24(val.to_u32().unwrap()),
                    }))
                }
                located::Constant::Tuple(_) => todo!(),
                located::Constant::Float(_) => todo!(),
                located::Constant::Complex { real: _, imag: _ } => todo!(),
                located::Constant::Ellipsis => todo!(),
            },

            rExpr::Name(n) => Some(FromExpr::Expr(imp::Expr::Var {
                nam: Name::new(n.id.to_string()),
            })),

            rExpr::Call(c) => {
                let fun = c.func;

                let expr = self.parse_expr_type(fun);

                if let Some(FromExpr::Expr(Expr::Var { ref nam })) = expr {
                    let mut args: Vec<Expr> = vec![];

                    for arg in c.args {
                        let arg = self.parse_expr_type(Box::new(arg));

                        if let Some(FromExpr::Expr(e)) = arg {
                            args.push(e);
                        }
                    }
                    if let Some(val) = self.book.adts.get(&nam.clone()) {
                        return Some(FromExpr::Expr(imp::Expr::Constructor {
                            name: val.ctrs.first().unwrap().0.clone(),
                            args,
                            kwargs: vec![],
                        }));
                    }
                    return Some(FromExpr::Expr(imp::Expr::Call {
                        fun: Box::new(Expr::Var {
                            nam: Name::new(nam.to_string()),
                        }),
                        args,
                        kwargs: vec![],
                    }));
                }
                expr
            }
            _ => todo!(),
        }
    }

    fn parse_bin_op(
        &self,
        bin: ExprBinOp<TextRange>,
        vars: Option<&Vec<String>>,
        subs: Option<&Vec<String>>,
    ) -> Option<FromExpr> {
        // TODO(#5): Treat case where expr type returns None

        let left: FromExpr;
        let right: FromExpr;

        if let (Some(vars), Some(subs)) = (vars, subs) {
            left = self.parse_expr_type_with_sub(bin.left, vars, subs).unwrap();
            right = self
                .parse_expr_type_with_sub(bin.right, vars, subs)
                .unwrap();
        } else {
            left = self.parse_expr_type(bin.left).unwrap();
            right = self.parse_expr_type(bin.right).unwrap();
        }
        let op: Op = match bin.op {
            rOperator::Add => Op::ADD,
            rOperator::Sub => Op::SUB,
            rOperator::Mult => Op::MUL,
            rOperator::MatMult => todo!(),
            rOperator::Div => Op::DIV,
            rOperator::Mod => todo!(),
            rOperator::Pow => Op::POW,
            rOperator::LShift => Op::SHL,
            rOperator::RShift => Op::SHR,
            rOperator::BitOr => Op::OR,
            rOperator::BitXor => Op::XOR,
            rOperator::BitAnd => Op::AND,
            rOperator::FloorDiv => todo!(),
        };

        if let (
            FromExpr::Expr(Expr::Var { nam: nam_l }),
            FromExpr::Expr(Expr::Var { nam: nam_r }),
        ) = (left.clone(), right.clone())
        {
            let adt_l = self.book.adts.get(&nam_l);
            let adt_r = self.book.adts.get(&nam_r);

            let mut is_adt = false;

            match (adt_l, adt_r) {
                (None, None) => {}
                (_, _) => {
                    is_adt = true;
                }
            }

            let mut fields: Vec<CtrField> = vec![];

            if is_adt {
                fields.push(CtrField {
                    nam: nam_l.clone(),
                    rec: false,
                });
            }

            return Some(FromExpr::CtrField(fields));
        }

        if let (FromExpr::Expr(left), FromExpr::Expr(right)) = (left, right) {
            let operation = imp::Expr::Bin {
                op,
                lhs: Box::new(left),
                rhs: Box::new(right),
            };

            return Some(FromExpr::Expr(operation));
        }
        todo!()
    }

    fn parse_assign(
        &mut self,
        assign: &StmtAssign<TextRange>,
    ) -> Option<FromExpr> {
        self.parse_expr_type(assign.value.clone())
    }

    fn parse_match(
        &mut self,
        m: &StmtMatch<TextRange>,
        stmts: &Vec<rStmt<TextRange>>,
        index: &usize,
    ) -> Option<imp::Stmt> {
        let mut arms: Vec<imp::MatchArm> = vec![];
        let mut patt: Vec<String> = vec![];

        let subj = self.parse_expr_type(m.subject.clone());
        let mut name: Option<String> = None;

        if let Some(FromExpr::Expr(subj)) = subj {
            name = match subj.clone() {
                Expr::Var { nam } => Some(nam.to_string()),
                _ => None,
            };
        }

        for case in &m.cases {
            let pat = match &case.pattern {
                rPattern::MatchValue(val) => {
                    let expr = self.parse_expr_type(val.value.clone()).unwrap();
                    match expr {
                        FromExpr::Expr(imp::Expr::Var { nam }) => Some(nam),
                        _ => None,
                    }
                }
                rPattern::MatchSingleton(_) => todo!(),
                rPattern::MatchSequence(_) => todo!(),
                rPattern::MatchMapping(_) => todo!(),
                rPattern::MatchClass(class) => {
                    let expr = self.parse_expr_type(class.cls.clone()).unwrap();

                    for val in class.patterns.clone() {
                        if let rPattern::MatchAs(match_as) = val {
                            if let Some(name) = &match_as.name {
                                patt.push(name.to_string());
                            }
                        }
                    }

                    match expr {
                        FromExpr::Expr(imp::Expr::Var { nam }) => Some(nam),
                        _ => None,
                    }
                }
                rPattern::MatchStar(_) => todo!(),
                rPattern::MatchAs(_) => todo!(),
                rPattern::MatchOr(_) => todo!(),
            };

            let stmt_arm = self.parse_vec_with_sub(
                &case.body.clone(),
                0,
                &patt,
                &vec![name.clone().unwrap()],
            );

            if let Some(pat) = pat {
                let first = Some(Name::new(format!("{}/{}", "Tree", pat)));
                if let Some(FromExpr::Statement(a)) = stmt_arm {
                    let arm = MatchArm { lft: first, rgt: a };
                    arms.push(arm);
                }
            }
        }

        if let Some(FromExpr::Expr(subj)) =
            self.parse_expr_type(m.subject.clone())
        {
            let nxt = self.parse_vec(stmts, index + 1);

            let my_nxt: Option<Box<Stmt>> = match nxt {
                Some(FromExpr::Statement(s)) => Some(Box::new(s)),
                _ => None,
            };

            let name = match subj.clone() {
                Expr::Var { nam } => Some(nam),
                _ => None,
            };

            let ret_match = imp::Stmt::Match {
                arg: Box::new(subj.clone()),
                // TODO(#7): Add binding
                bind: name,
                arms,
                nxt: my_nxt,
            };

            return Some(ret_match);
        }
        None
    }

    fn parse_vec_with_sub(
        &mut self,
        stmts: &Vec<rStmt<TextRange>>,
        index: usize,
        vars: &Vec<String>,
        subs: &Vec<String>,
    ) -> Option<FromExpr> {
        let stmt = match stmts.get(index) {
            Some(s) => s,
            None => {
                return None;
            }
        };

        match stmt {
            rStmt::Assign(assign) => {
                let value = self.parse_assign(assign).unwrap();
                // TODO: Fix this first use
                let mut name = assign
                    .targets
                    .first()
                    .unwrap()
                    .clone()
                    .name_expr()
                    .unwrap()
                    .id
                    .to_string();

                for (index, var) in vars.iter().enumerate() {
                    if *var == name {
                        name = format!("{}.{}", name, subs.get(index).unwrap());
                    }
                }

                let nxt = self.parse_vec_with_sub(stmts, index + 1, vars, subs);

                if let FromExpr::Expr(Expr::Call {
                    fun,
                    args: _,
                    kwargs: _,
                }) = value.clone()
                {
                    if let Expr::Var { nam } = *fun {
                        #[allow(clippy::cmp_owned)]
                        if nam.to_string() == "switch" {
                            let mut arms: Vec<imp::Stmt> = vec![];
                            if let Some(rStmt::Match(m)) = stmts.get(index + 1)
                            {
                                for case in &m.cases {
                                    let stmt_arm = self.parse_vec_with_sub(
                                        &case.body.clone(),
                                        0,
                                        vars,
                                        subs,
                                    );

                                    if let Some(FromExpr::Statement(a)) =
                                        stmt_arm
                                    {
                                        arms.push(a);
                                    }
                                }

                                if let Some(FromExpr::Expr(expr)) = self
                                    .parse_expr_type_with_sub(
                                        m.subject.clone(),
                                        vars,
                                        subs,
                                    )
                                {
                                    return Some(FromExpr::Statement(
                                        imp::Stmt::Switch {
                                            arg: Box::new(expr),
                                            bind: Some(Name::new(name)),
                                            arms,
                                            nxt: nxt.map(|n| {
                                                if let FromExpr::Statement(n) =
                                                    n
                                                {
                                                    return Box::new(n);
                                                }

                                                todo!()
                                            }),
                                        },
                                    ));
                                }
                            }
                        }
                    }
                }

                if let FromExpr::Expr(val) = value {
                    return Some(FromExpr::Statement(imp::Stmt::Assign {
                        pat: imp::AssignPattern::Var(Name::new(name)),
                        val: Box::new(val),
                        nxt: nxt.map(|n| {
                            if let FromExpr::Statement(n) = n {
                                return Box::new(n);
                            }

                            todo!()
                        }),
                    }));
                }

                Some(value)
            }
            rStmt::If(stmt_if) => {
                let cond = self.parse_expr_type(stmt_if.test.clone());
                let then =
                    self.parse_vec_with_sub(&stmt_if.body, 0, vars, subs);
                let otherwise =
                    self.parse_vec_with_sub(&stmt_if.orelse, 0, vars, subs);

                let nxt = self.parse_vec_with_sub(stmts, index + 1, vars, subs);

                let b_nxt = match nxt {
                    Some(FromExpr::Statement(nxt)) => Some(Box::new(nxt)),
                    _ => None,
                };

                match (cond, then, otherwise, b_nxt) {
                    (
                        Some(FromExpr::Expr(cond)),
                        Some(FromExpr::Statement(then)),
                        Some(FromExpr::Statement(otherwise)),
                        b_nxt,
                    ) => Some(FromExpr::Statement(Stmt::If {
                        cond: Box::new(cond),
                        then: Box::new(then),
                        otherwise: Box::new(otherwise),
                        nxt: b_nxt,
                    })),
                    (_, _, _, _) => {
                        panic!("If Statement must have an else.")
                    }
                }
            }
            rStmt::Return(r) => match &r.value {
                Some(val) => {
                    let term = self
                        .parse_expr_type_with_sub(val.clone(), vars, subs)
                        .unwrap();
                    if let FromExpr::Expr(term) = term {
                        return Some(FromExpr::Statement(imp::Stmt::Return {
                            term: Box::new(term),
                        }));
                    }

                    todo!()
                }
                None => None,
            },
            rStmt::Match(m) => {
                println!("Match: {:?}", m);

                if let Some(val) = self.parse_match(m, stmts, &index) {
                    return Some(FromExpr::Statement(val));
                }

                None
            }
            _ => None,
        }
    }

    fn parse_vec(
        &mut self,
        stmts: &Vec<rStmt<TextRange>>,
        index: usize,
    ) -> Option<FromExpr> {
        let stmt = match stmts.get(index) {
            Some(s) => s,
            None => {
                return None;
            }
        };

        match stmt {
            rStmt::Assign(assign) => {
                let value = self.parse_assign(assign).unwrap();
                let name = assign
                    .targets
                    .first()
                    .unwrap()
                    .clone()
                    .name_expr()
                    .unwrap()
                    .id
                    .to_string();

                let nxt = self.parse_vec(stmts, index + 1);

                if let FromExpr::Expr(Expr::Call {
                    fun,
                    args: _,
                    kwargs: _,
                }) = value.clone()
                {
                    if let Expr::Var { nam } = *fun {
                        #[allow(clippy::cmp_owned)]
                        if nam.to_string() == "switch" {
                            let mut arms: Vec<imp::Stmt> = vec![];
                            if let Some(rStmt::Match(m)) = stmts.get(index + 1)
                            {
                                for case in &m.cases {
                                    let stmt_arm =
                                        self.parse_vec(&case.body.clone(), 0);

                                    if let Some(FromExpr::Statement(a)) =
                                        stmt_arm
                                    {
                                        arms.push(a);
                                    }
                                }

                                if let Some(FromExpr::Expr(expr)) =
                                    self.parse_expr_type(m.subject.clone())
                                {
                                    return Some(FromExpr::Statement(
                                        imp::Stmt::Switch {
                                            arg: Box::new(expr),
                                            bind: Some(Name::new(name)),
                                            arms,
                                            nxt: nxt.map(|n| {
                                                if let FromExpr::Statement(n) =
                                                    n
                                                {
                                                    return Box::new(n);
                                                }

                                                todo!()
                                            }),
                                        },
                                    ));
                                }
                            }
                        }
                    }
                }

                if let FromExpr::Expr(val) = value {
                    return Some(FromExpr::Statement(imp::Stmt::Assign {
                        pat: imp::AssignPattern::Var(Name::new(name)),
                        val: Box::new(val),
                        nxt: nxt.map(|n| {
                            if let FromExpr::Statement(n) = n {
                                return Box::new(n);
                            }

                            todo!()
                        }),
                    }));
                }

                Some(value)
            }
            rStmt::If(stmt_if) => {
                let cond = self.parse_expr_type(stmt_if.test.clone());
                let then = self.parse_vec(&stmt_if.body, 0);
                let otherwise = self.parse_vec(&stmt_if.orelse, 0);

                let nxt = self.parse_vec(stmts, index + 1);

                let b_nxt = match nxt {
                    Some(FromExpr::Statement(nxt)) => Some(Box::new(nxt)),
                    _ => None,
                };

                match (cond, then, otherwise, b_nxt) {
                    (
                        Some(FromExpr::Expr(cond)),
                        Some(FromExpr::Statement(then)),
                        Some(FromExpr::Statement(otherwise)),
                        b_nxt,
                    ) => Some(FromExpr::Statement(Stmt::If {
                        cond: Box::new(cond),
                        then: Box::new(then),
                        otherwise: Box::new(otherwise),
                        nxt: b_nxt,
                    })),
                    (_, _, _, _) => {
                        panic!("If Statement must have an else.")
                    }
                }
            }
            rStmt::Return(r) => match &r.value {
                Some(val) => {
                    let term = self.parse_expr_type(val.clone()).unwrap();
                    if let FromExpr::Expr(term) = term {
                        return Some(FromExpr::Statement(imp::Stmt::Return {
                            term: Box::new(term),
                        }));
                    }

                    todo!()
                }
                None => None,
            },
            rStmt::Match(m) => {
                println!("Match: {:?}", m);

                if let Some(val) = self.parse_match(m, stmts, &index) {
                    return Some(FromExpr::Statement(val));
                }

                None
            }
            _ => None,
        }
    }

    fn add_adt(&mut self, nam: Name, adt: Adt) {
        if let Some(adt) = self.book.adts.get(&nam) {
            if adt.builtin {
                panic!(
                    "{} is a built-in datatype and should not be overridden.",
                    nam
                );
            } else {
                panic!("Repeated datatype '{}'", nam);
            }
        } else {
            for ctr in adt.ctrs.keys() {
                match self.book.ctrs.entry(ctr.clone()) {
                    indexmap::map::Entry::Vacant(e) => {
                        _ = e.insert(nam.clone());
                    }
                    indexmap::map::Entry::Occupied(e) => {
                        if self
                            .book
                            .adts
                            .get(e.get())
                            .is_some_and(|adt| adt.builtin)
                        {
                            panic!(
                                "{} is a built-in constructor and should not be overridden.",
                                e.key()
                            );
                        } else {
                            panic!("Repeated constructor '{}'", e.key());
                        }
                    }
                }
            }
        }
        self.book.adts.insert(nam.clone(), adt);
    }

    pub fn parse(&mut self, _fun: &str) -> String {
        for stmt in self.statements.clone() {
            match stmt {
                rStmt::FunctionDef(fun_def) => {
                    let args = *fun_def.args;
                    let mut names: Vec<Name> = vec![];

                    for arg in args.args {
                        names.push(Name::new(arg.def.arg.to_string()));
                    }

                    let expr = self.parse_vec(&fun_def.body, 0);

                    if let Some(FromExpr::Statement(e)) = expr {
                        let def = imp::Definition {
                            name: Name::new(fun_def.name.to_string()),
                            params: names,
                            body: e,
                        };
                        self.definitions.push(def);
                    }
                }
                rStmt::Assign(assign) => {
                    let iden = assign.targets.first().unwrap();

                    let name: String;

                    if let rExpr::Name(iden) = iden {
                        name = iden.id.to_string();
                        let mut adt = Adt {
                            ctrs: IndexMap::new(),
                            builtin: false,
                        };

                        let body = self.parse_expr_type(assign.value);

                        if let Some(FromExpr::CtrField(ctr)) = body {
                            adt.ctrs.insert(
                                Name::new(format!(
                                    "{}/{}",
                                    name,
                                    ctr.first().unwrap().nam
                                )),
                                ctr,
                            );
                        }

                        self.add_adt(Name::new(name), adt);
                    }
                }
                rStmt::ClassDef(class) => {
                    let is_dataclass = class.decorator_list.iter().any(|exp| {
                        if let rExpr::Name(nam) = exp {
                            if nam.id.to_string() == "dataclass" {
                                return true;
                            }
                        }
                        false
                    });

                    let iden = class.name.to_string();
                    let mut adt = Adt {
                        ctrs: IndexMap::new(),
                        builtin: false,
                    };

                    if is_dataclass {
                        for stmt in class.body {
                            match stmt {
                                rStmt::AnnAssign(assign) => {
                                    //let mut e_type = String::default();
                                    let mut target = String::default();

                                    //if let rExpr::Name(nam) = *assign.annotation.clone() {
                                    //    e_type = nam.id.to_string();
                                    //}

                                    if let rExpr::Name(nam) =
                                        *assign.target.clone()
                                    {
                                        target = nam.id.to_string();
                                    }

                                    let ctr_field = CtrField {
                                        nam: Name::new(target),
                                        rec: true,
                                    };

                                    adt.ctrs.insert(
                                        Name::new(iden.to_string()),
                                        vec![ctr_field],
                                    );
                                }
                                _ => todo!(),
                            }
                        }
                        self.add_adt(Name::new(iden.clone()), adt);
                    }
                }
                _ => {
                    //self.parse_part(self.statements.get(self.index).unwrap().clone());
                }
            }
        }

        for def in &self.definitions {
            let fun_def = def.clone().to_fun(false).unwrap();
            self.book.defs.insert(fun_def.name.clone(), fun_def.clone());
        }

        let main_def = imp::Definition {
            name: Name::new("main"),
            params: vec![],
            body: imp::Stmt::Return {
                term: Box::new(imp::Expr::Call {
                    fun: Box::new(imp::Expr::Var {
                        nam: Name::new("sum_tree"),
                    }),
                    args: vec![
                        imp::Expr::Num {
                            val: bend::fun::Num::U24(10),
                        },
                        imp::Expr::Num {
                            val: bend::fun::Num::U24(5),
                        },
                    ],
                    kwargs: vec![],
                }),
            },
        };

        self.book
            .defs
            .insert(Name::new("main"), main_def.to_fun(true).unwrap());

        self.book.entrypoint = None;

        println!("BEND:\n {}", self.book.display_pretty());
        println!("\n\nADTS:\n {:?}\n\n", self.book.adts);

        let return_val = run(&self.book);

        match return_val {
            Some(val) => val.0.to_string(),
            None => panic!("Could not run Bend code."),
        }
    }
}
