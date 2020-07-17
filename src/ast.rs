// https://message-from-space.readthedocs.io/en/latest/message7.html

use std::cmp::max;

type BSymbol = Box<Symbol>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Symbol {
    Lit(i64),          // 1-3
    Eq,                // 4
    Inc,               // 5
    Dec,               // 6
    Add,               // 7
    Var(usize),        // 8
    Mul,               // 9
    Div,               // 10
    T,                 // 11 & 21
    F,                 // 11 & 22
    Lt,                // 12
    Mod,               // 13
    Dem,               // 14
    Send,              // 15
    Neg,               // 16
    Ap,                // 17
    S,                 // 18
    C,                 // 19
    B,                 // 20
    Pwr2,              // 23
    I,                 // 24
    Cons,              // 25
    Car,               // 26
    Cdr,               // 27
    Nil,               // 28
    IsNil,             // 29
    List(Vec<Symbol>), // 30
    // 31 .. vec = alias for cons that looks nice in “vector” usage context.
    Draw,         // 32
    Checkerboard, // 33
    MultipleDraw, // 34
    // 35 = modulate list, doesn't seem to map to an operation
    // 36 = send 0:
    //   :1678847
    //   ap send ( 0 )   =   ( 1 , :1678847 )
    If0,      // 37
    Interact, // 38
    // 39 = interaction protocol
    StatelessDraw,
}

pub fn eval_instructions(tree: &[Symbol]) -> Symbol {
    let len = max_vars(&tree);
    let mut vars = vec![Symbol::Nil; len];

    eval(tree, &mut vars).0.clone()
}

fn max_vars(instructions: &[Symbol]) -> usize {
    instructions.iter().fold(0 as usize, |acc, el| match el {
        Symbol::Var(idx) => max(*idx, acc),
        _ => acc,
    })
}

fn eval<'a>(instructions: &'a [Symbol], vars: &mut Vec<Symbol>) -> (Symbol, &'a [Symbol]) {
    let (op, rest) = instructions.split_first().unwrap();
    match op {
        Symbol::Lit(_) => (op.clone(), rest),
        Symbol::Eq => {
            let (lhs, rest0) = eval(rest, vars);
            let (rhs, rest1) = eval(rest0, vars);
            (if lhs == rhs { Symbol::T } else { Symbol::F }, rest1)
        }
        _ => unimplemented!("{0:?} is not implemented", op),
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::{eval_instructions, Symbol};

    #[test]
    fn equality() {
        let res = eval_instructions(&[Symbol::Eq, Symbol::Lit(1), Symbol::Lit(1)]);
        assert_eq!(res, Symbol::T);
    }

    #[test]
    fn inequality() {
        let res = eval_instructions(&[Symbol::Eq, Symbol::Lit(1), Symbol::Lit(2)]);
        assert_eq!(res, Symbol::F);
    }
}
