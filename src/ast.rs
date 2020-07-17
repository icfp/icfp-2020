// https://message-from-space.readthedocs.io/en/latest/message7.html

use std::cmp::max;

pub enum Symbol {
    Number(i16), // 1-3
    Eq(Box<Symbol>, Box<Symbol>), // 4
    Inc(Box<Symbol>), // 5
    Dec(Box<Symbol>), // 6
    Add(Box<Symbol>, Box<Symbol>), // 7
    Var(usize), // 8
    Mul(Box<Symbol>, Box<Symbol>), // 9
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
    }
}