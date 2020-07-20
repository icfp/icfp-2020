use super::ast::Symbol::*;
use crate::parser::parse_as_lines;

#[test]
fn run_inc_1() {
    let symbol = super::run(":1096 = ap inc 1");
    dbg!(&symbol);

    assert_eq!(symbol, Lit(2))
}

#[test]
fn run_inc_var() {
    let symbol = super::run(
        ":1096 = ap inc 1
         :302 = ap inc :1096",
    );
    dbg!(&symbol);

    assert_eq!(symbol, Lit(3))
}

#[test]
#[ignore]
fn test_lookahead() {
    let symbol = super::run(
        ":1 = ap add 1
                                     :2 = ap :1 2",
    );
    dbg!(&symbol);

    assert_eq!(symbol, Lit(3))
}

#[test]
fn test_laziness() {
    let symbol = super::run(":1 = ap ap ap if0 1 :1 3");
    dbg!(&symbol);

    assert_eq!(symbol, Lit(3))
}

#[test]
fn run_simple_add() {
    let symbol = super::run(":1 = ap ap add 1 2");
    dbg!(&symbol);

    assert_eq!(symbol, Lit(3))
}

#[test]
fn run_simple() {
    let symbol = super::run(
        ":1 = ap add 1
:2 = ap ap ap ap if0 1 :2 :1 2
:3 = :2",
    );
    dbg!(&symbol);

    assert_eq!(symbol, Lit(3))
}

#[test]
fn run_start() {
    let symbol = super::run(
        ":1029 = ap ap cons 7 ap ap cons 123229502148636 nil
:1030 = ap ap cons 2 ap ap cons 7 nil
:1031 = ap ap cons 4 ap ap cons 21855 nil
:1032 = ap ap cons 7 ap ap cons 560803991675135 nil
:1034 = ap ap cons 5 ap ap cons 33554431 nil
:1035 = ap ap cons 5 ap ap cons 30309607 nil
:1036 = ap ap cons 3 ap ap cons 463 nil
:1037 = ap ap cons 4 ap ap cons 48063 nil
:1038 = ap ap cons 7 ap ap cons 10880 nil
:1039 = ap ap cons 5 ap ap cons 15265326 nil
:1040 = ap ap cons 5 ap ap cons 18472561 nil
:1041 = ap ap cons 4 ap ap cons 64959 nil
:1042 = ap ap cons 4 ap ap cons 63935 nil",
    );
    dbg!(&symbol);

    assert_eq!(
        symbol,
        Pair(
            Lit(4).into(),
            Closure {
                captured_arg: Nil.into(),
                body: Closure {
                    captured_arg: Lit(63935).into(),
                    body: Cons.into()
                }
                .into()
            }
            .into()
        )
    )
}

#[test]
fn run_galaxy_stack() {
    let mut lines = super::parser::parse_as_lines(include_str!("../data/galaxy.txt"));
    // ap ap ap interact x0 nil ap ap vec 0 0 = ( x16 , ap multipledraw x64 )
    // ap ap ap interact x0 x16 ap ap vec x1 x2 = ( x17 , ap multipledraw x65 )
    // ap ap ap interact x0 x17 ap ap vec x3 x4 = ( x18 , ap multipledraw x66 )
    // ap ap ap interact x0 x18 ap ap vec x5 x6 = ( x19 , ap multipledraw x67 )
    lines.extend_from_slice(&parse_as_lines("run = ap ap interact galaxy nil ( 0, 0 )"));

    super::stack_interpreter::stack_interpret(lines);
}
