// https://message-from-space.readthedocs.io/en/latest/message7.html

use std::cmp::max;

pub enum Symbol {
    Number(i16),                              // 1-3
    Eq(Box<Symbol>, Box<Symbol>),             // 4
    Inc(Box<Symbol>),                         // 5
    Dec(Box<Symbol>),                         // 6
    Add(Box<Symbol>, Box<Symbol>),            // 7
    Var(usize),                               // 8
    Mul(Box<Symbol>, Box<Symbol>),            // 9
    Div(Box<Symbol>, Box<Symbol>),            // 10
    T(Box<Symbol>, Box<Symbol>),              // 11 & 21
    F(Box<Symbol>, Box<Symbol>),              // 11 & 22
    Lt(Box<Symbol>, Box<Symbol>),             // 12
    Mod(Box<Symbol>),                         // 13
    Dem(Box<Symbol>),                         // 14
    Send(Box<Symbol>),                        // 15
    Neg(Box<Symbol>),                         // 16
    Ap(Box<Symbol>, Box<Symbol>),             // 17
    S(Box<Symbol>, Box<Symbol>, Box<Symbol>), // 18
    C(Box<Symbol>, Box<Symbol>, Box<Symbol>), // 19
    B(Box<Symbol>, Box<Symbol>, Box<Symbol>), // 20
    Pwr2(Box<Symbol>),                        // 23
    I(Box<Symbol>),                           // 24
}

pub fn eval_tree(tree: Symbol) -> i64 {
    let len = max_vars(&tree);
    let mut vars = vec![0 as i64; len];

    eval(tree, &mut vars)
}

fn max_vars(tree: &Symbol) -> usize {
    match tree {
        Symbol::Number(_) => 0,
        Symbol::Eq(x, y) => max(max_vars(x), max_vars(y)),
        Symbol::Inc(x) => max_vars(x),
        Symbol::Dec(x) => max_vars(x),
        Symbol::Add(x, _) => max_vars(x),
        Symbol::Var(x) => *x,
        Symbol::Mul(x, y) => max(max_vars(x), max_vars(y)),
        Symbol::Div(_, _) => unimplemented!("Div is not implemented"),
        Symbol::T(_, _) => unimplemented!("T is not implemented"),
        Symbol::F(_, _) => unimplemented!("F is not implemented"),
        Symbol::Lt(_, _) => unimplemented!("Lt is not implemented"),
        Symbol::Mod(_) => unimplemented!("Mod is not implemented"),
        Symbol::Dem(_) => unimplemented!("Dem is not implemented"),
        Symbol::Send(_) => unimplemented!("Send is not implemented"),
        Symbol::Neg(_) => unimplemented!("Neg is not implemented"),
        Symbol::Ap(_, _) => unimplemented!("Ap is not implemented"),
        Symbol::S(_, _, _) => unimplemented!("S is not implemented"),
        Symbol::C(_, _, _) => unimplemented!("C is not implemented"),
        Symbol::B(_, _, _) => unimplemented!("B is not implemented"),
        Symbol::Pwr2(_) => unimplemented!("Pwr2 is not implemented"),
        Symbol::I(_) => unimplemented!("I is not implemented"),
    }
}

fn eval(tree: Symbol, vars: &mut Vec<i64>) -> i64 {
    match tree {
        Symbol::Number(i) => i as i64,
        Symbol::Eq(x, y) => (eval(*x, vars) == eval(*y, vars)) as i64,
        Symbol::Inc(x) => eval(*x, vars) + 1,
        Symbol::Dec(x) => eval(*x, vars) - 1,
        Symbol::Add(x, y) => eval(*x, vars) + eval(*y, vars),
        Symbol::Var(x) => vars[x],
        Symbol::Mul(x, y) => eval(*x, vars) * eval(*y, vars),
        Symbol::Div(_, _) => unimplemented!("Div is not implemented"),
        Symbol::T(_, _) => unimplemented!("T is not implemented"),
        Symbol::F(_, _) => unimplemented!("F is not implemented"),
        Symbol::Lt(_, _) => unimplemented!("Lt is not implemented"),
        Symbol::Mod(_) => unimplemented!("Mod is not implemented"),
        Symbol::Dem(_) => unimplemented!("Dem is not implemented"),
        Symbol::Send(_) => unimplemented!("Send is not implemented"),
        Symbol::Neg(_) => unimplemented!("Neg is not implemented"),
        Symbol::Ap(_, _) => unimplemented!("Ap is not implemented"),
        Symbol::S(_, _, _) => unimplemented!("S is not implemented"),
        Symbol::C(_, _, _) => unimplemented!("C is not implemented"),
        Symbol::B(_, _, _) => unimplemented!("B is not implemented"),
        Symbol::Pwr2(_) => unimplemented!("Pwr2 is not implemented"),
        Symbol::I(_) => unimplemented!("I is not implemented"),
    }
}
