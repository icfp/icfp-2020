use crate::ast::{eval_instructions, Symbol};

#[test]
fn equality() {
    let res = eval_instructions(&[
        Symbol::Ap,
        Symbol::Ap,
        Symbol::Eq,
        Symbol::Lit(1),
        Symbol::Lit(1),
    ]);
    assert_eq!(res, Symbol::T);
}

#[test]
fn inequality() {
    let res = eval_instructions(&[
        Symbol::Ap,
        Symbol::Ap,
        Symbol::Eq,
        Symbol::Lit(1),
        Symbol::Lit(2),
    ]);
    assert_eq!(res, Symbol::F);
}

#[test]
fn message5() {
    // from https://message-from-space.readthedocs.io/en/latest/message5.html

    let res = eval_instructions(&[Symbol::Ap, Symbol::Inc, Symbol::Lit(0)]);
    assert_eq!(res, Symbol::Lit(1));

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
}
