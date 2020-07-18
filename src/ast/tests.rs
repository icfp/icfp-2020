use std::collections::HashMap;

use crate::ast::Symbol::*;
use crate::ast::{eval, eval_instructions, Symbol};

#[test]
fn test_modulate() {
    fn val(s: &str) -> Symbol {
        StringValue(s.to_string())
    }

    assert_eq!(eval_instructions(&[Ap, Mod, Lit(0)]), val("010"));
    assert_eq!(eval_instructions(&[Ap, Mod, Lit(1)]), val("01100001"));
    assert_eq!(eval_instructions(&[Ap, Mod, Lit(-1)]), val("10100001"));
    assert_eq!(
        eval_instructions(&[Ap, Mod, Lit(256)]),
        val("011110000100000000")
    );
}

#[test]
fn test_demodulate() {
    assert_eq!(eval_instructions(&[Ap, Dem, Ap, Mod, Lit(0)]), Lit(0));
    assert_eq!(eval_instructions(&[Ap, Dem, Ap, Mod, Lit(1)]), Lit(1));
    assert_eq!(eval_instructions(&[Ap, Dem, Ap, Mod, Lit(-1)]), Lit(-1));
    assert_eq!(eval_instructions(&[Ap, Dem, Ap, Mod, Lit(256)]), Lit(256));
    assert_eq!(eval_instructions(&[Ap, Dem, Ap, Mod, Lit(-256)]), Lit(-256));
}

#[test]
fn equality() {
    let res = eval_instructions(&[Ap, Ap, Eq, Lit(1), Lit(1)]);
    assert_eq!(res, T);
}

#[test]
fn inequality() {
    let res = eval_instructions(&[Ap, Ap, Eq, Lit(1), Lit(2)]);
    assert_eq!(res, F);
}

#[test]
fn message5() {
    // from https://message-from-space.readthedocs.io/en/latest/message5.html

    /*
    ap inc 0   =   1
    ap inc 1   =   2
    ap inc 2   =   3
    ap inc 3   =   4
    ap inc 300   =   301
    ap inc 301   =   302
    ap inc -1   =   0
    ap inc -2   =   -1
    ap inc -3   =   -2
    */

    let res = eval_instructions(&[Ap, Inc, Lit(0)]);
    assert_eq!(res, Lit(1));

    let res = eval_instructions(&[Ap, Inc, Lit(1)]);
    assert_eq!(res, Lit(2));

    let res = eval_instructions(&[Ap, Inc, Lit(2)]);
    assert_eq!(res, Lit(3));

    let res = eval_instructions(&[Ap, Inc, Lit(3)]);
    assert_eq!(res, Lit(4));

    let res = eval_instructions(&[Ap, Inc, Lit(300)]);
    assert_eq!(res, Lit(301));

    let res = eval_instructions(&[Ap, Inc, Lit(301)]);
    assert_eq!(res, Lit(302));

    let res = eval_instructions(&[Ap, Inc, Lit(-1)]);
    assert_eq!(res, Lit(0));

    let res = eval_instructions(&[Ap, Inc, Lit(-2)]);
    assert_eq!(res, Lit(-1));

    let res = eval_instructions(&[Ap, Inc, Lit(-3)]);
    assert_eq!(res, Lit(-2));
}

#[test]
fn message9() {
    // https://message-from-space.readthedocs.io/en/latest/message9.html

    /*
    ap ap mul 4 2   =   8
    ap ap mul 3 4   =   12
    ap ap mul 3 -2   =   -6
    ap ap mul x0 x1   =   ap ap mul x1 x0
    ap ap mul x0 0   =   0
    ap ap mul x0 1   =   x0
    */

    let res = eval_instructions(&[Ap, Ap, Mul, Lit(4), Lit(2)]);
    assert_eq!(res, Lit(8));

    let res = eval_instructions(&[Ap, Ap, Mul, Lit(3), Lit(4)]);
    assert_eq!(res, Lit(12));

    let res = eval_instructions(&[Ap, Ap, Mul, Lit(3), Lit(-2)]);
    assert_eq!(res, Lit(-6));

    let res = eval(
        &[Ap, Ap, Mul, Var(0), Var(1)],
        &mut vec![(0, Lit(42)), (1, Lit(7))].into_iter().collect(),
    );

    assert_eq!(res, Lit(294));

    let res = eval(
        &[Ap, Ap, Mul, Var(0), Lit(0)],
        &mut vec![(0, Lit(42))].into_iter().collect(),
    );

    assert_eq!(res, Lit(0));

    let res = eval(
        &[Ap, Ap, Mul, Var(0), Lit(1)],
        &mut vec![(0, Lit(42))].into_iter().collect(),
    );

    assert_eq!(res, Lit(42));
}

#[test]
fn message10() {
    // https://message-from-space.readthedocs.io/en/latest/message10.html

    /*
    ap ap div 4 2   =   2
    ap ap div 4 3   =   1
    ap ap div 4 4   =   1
    ap ap div 4 5   =   0
    ap ap div 5 2   =   2
    ap ap div 6 -2   =   -3
    ap ap div 5 -3   =   -1
    ap ap div -5 3   =   -1
    ap ap div -5 -3   =   1
    ap ap div x0 1   =   x0
    */

    let res = eval_instructions(&[Ap, Ap, Div, Lit(4), Lit(2)]);
    assert_eq!(res, Lit(2));

    let res = eval_instructions(&[Ap, Ap, Div, Lit(4), Lit(3)]);
    assert_eq!(res, Lit(1));

    let res = eval_instructions(&[Ap, Ap, Div, Lit(4), Lit(4)]);
    assert_eq!(res, Lit(1));

    let res = eval_instructions(&[Ap, Ap, Div, Lit(4), Lit(5)]);
    assert_eq!(res, Lit(0));

    let res = eval_instructions(&[Ap, Ap, Div, Lit(5), Lit(2)]);
    assert_eq!(res, Lit(2));

    let res = eval_instructions(&[Ap, Ap, Div, Lit(6), Lit(-2)]);
    assert_eq!(res, Lit(-3));

    let res = eval_instructions(&[Ap, Ap, Div, Lit(5), Lit(-3)]);
    assert_eq!(res, Lit(-1));

    let res = eval_instructions(&[Ap, Ap, Div, Lit(-5), Lit(-3)]);
    assert_eq!(res, Lit(1));

    let res = eval(
        &[Ap, Ap, Div, Var(0), Lit(1)],
        &mut vec![(0, Lit(42))].into_iter().collect(),
    );

    assert_eq!(res, Lit(42));
}

#[test]
fn message18() {
    /*
    ap ap ap s x0 x1 x2   =   ap ap x0 x2 ap x1 x2
    ap ap ap s add inc 1   =   3
    ap ap ap s mul ap add 1 6   =   42
    */

    // let res = eval(
    //     &[Ap, Ap, Ap, S, Div, Var(0), Lit(1)],
    //     &mut vec![(0, Lit(42))].into_iter().collect(),
    // );

    let res = eval_instructions(&[Ap, Ap, Ap, S, Add, Inc, Lit(1)]);
    assert_eq!(res, Lit(3));

    let res = eval_instructions(&[Ap, Ap, Ap, S, Mul, Ap, Add, Lit(1), Lit(6)]);
    assert_eq!(res, Lit(42));
}

#[test]
fn message21() {
    /*
    ap ap t x0 x1   =   x0
    ap ap t 1 5   =   1
    ap ap t t i   =   t
    ap ap t t ap inc 5   =   t
    ap ap t ap inc 5 t   =   6
    */

    let res = eval_instructions(&[Ap, Ap, T, Lit(1), Lit(5)]);
    assert_eq!(res, Lit(1));

    let res = eval_instructions(&[Ap, Ap, T, T, Lit(5)]);
    assert_eq!(res, T);

    let res = eval_instructions(&[Ap, Ap, T, T, Ap, Inc, Lit(5)]);
    assert_eq!(res, T);

    let res = eval_instructions(&[Ap, Ap, T, Ap, Inc, Lit(5), T]);
    assert_eq!(res, Lit(6));
}

#[test]
fn message37_is_0() {
    let res = eval(
        &[Ap, Ap, Ap, If0, Lit(0), Var(1), Lit(2)],
        &mut vec![(1, Lit(42))].into_iter().collect(),
    );

    assert_eq!(res, Lit(42))
}

#[test]
fn message37_is_not_0() {
    let res = eval_instructions(&[Ap, Ap, Ap, If0, Lit(1), Lit(0), Lit(1)]);

    assert_eq!(res, Symbol::Lit(1))
}
