use super::ast::Symbol::*;

#[test]
fn run_inc_1() {
    let symbol = super::run(":1096 = ap inc 1");
    dbg!(&symbol);

    assert_eq!(symbol, Lit(2))
}
