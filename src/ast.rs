// https://message-from-space.readthedocs.io/en/latest/message7.html

use std::cmp::max;
use std::collections::{HashMap, VecDeque};

type BSymbol = Box<Symbol>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Symbol {
    Lit(i64),          // 1-3
    Eq,                // 4
    Inc,               // 5
    Dec,               // 6
    Add,               // 7
    Get(usize),        // 8
    Set(usize),        // 8
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
    let mut vars = HashMap::new();

    eval(tree, &mut vars)
}

fn eval_fn(op: Symbol, operands: &mut VecDeque<Symbol>, vars: &mut HashMap<i32, Symbol>) -> Symbol {
    match op {
        Symbol::Eq => {
            // we're iterating backwards, operand order is reversed
            let rhs = operands.pop_back().unwrap();
            let lhs = operands.pop_back().unwrap();
            if lhs == rhs {
                Symbol::T
            } else {
                Symbol::F
            }
        }

        _ => unimplemented!("{0:?} is not implemented", op),
    }
}

fn eval(instructions: &[Symbol], vars: &mut HashMap<i32, Symbol>) -> Symbol {
    let mut stack = VecDeque::<Symbol>::new();
    for inst in instructions.iter().rev() {
        match inst {
            Symbol::Ap => {
                let op = stack.pop_back().unwrap();
                let res = eval_fn(op, &mut stack, vars);
                stack.push_back(res);
            }
            _ => stack.push_back(inst.clone()),
        }
    }

    assert_eq!(stack.len(), 1);

    stack.pop_back().unwrap()
}

#[cfg(test)]
mod tests {
    use crate::ast::{eval_instructions, Symbol};

    #[test]
    fn equality() {
        let res = eval_instructions(&[Symbol::Ap, Symbol::Eq, Symbol::Lit(1), Symbol::Lit(1)]);
        assert_eq!(res, Symbol::T);
    }

    #[test]
    fn inequality() {
        let res = eval_instructions(&[Symbol::Ap, Symbol::Eq, Symbol::Lit(1), Symbol::Lit(2)]);
        assert_eq!(res, Symbol::F);
    }
}
