use crate::ast::Statement;
use pest::Parser;

use crate::parser::{parse_as_lines, ProgramParser, Rule};

macro_rules! map (
    { $key:expr => $value:expr } => {
        {
            vec![Statement($key, $value)]
        }
     };
);

#[test]
fn parse_assignment() {
    // https://pest.rs/book/examples/csv.html#writing-the-parser

    let map = parse_as_lines(":1029 = ap ap cons 7 ap ap cons 123229502148636 nil");

    use crate::ast::Identifier;
    use crate::ast::Symbol::*;
    let expected = map!(
        Identifier::Var(1029) => vec![Ap, Ap, Cons, Lit(7), Ap, Ap, Cons, Lit(123229502148636), Nil]
    );

    assert_eq!(expected, map);
    println!("{:?}", map);
}

#[test]
fn parse_inc() {
    // https://pest.rs/book/examples/csv.html#writing-the-parser

    let map = parse_as_lines(":1029 = ap inc 300 nil");

    use crate::ast::Identifier;
    use crate::ast::Symbol::*;
    let expected = map!(
        Identifier::Var(1029) => vec![Ap, Inc, Lit(300), Nil]
    );
    assert_eq!(expected, map);
    println!("{:?}", map);
}

#[test]
fn parse_eq() {
    // https://pest.rs/book/examples/csv.html#writing-the-parser

    let map = parse_as_lines("t = ap ap eq :0 :0");

    use crate::ast::Identifier;
    use crate::ast::Symbol::*;
    let expected = map!(
        Identifier::Name("t".to_string()) => vec![Ap, Ap, Eq, Var(0), Var(0)]
    );

    assert_eq!(expected, map);
    println!("{:?}", map);
}

#[test]
fn parse_mod() {
    // https://pest.rs/book/examples/csv.html#writing-the-parser

    let map = parse_as_lines(":0 = ap mod 0");

    use crate::ast::Identifier;
    use crate::ast::Symbol::*;
    let expected = map!(
        Identifier::Var(0) => vec![Ap, Mod, Lit(0)]
    );

    assert_eq!(expected, map);
    println!("{:?}", map);
}

#[test]
fn parse_mod_with_negative() {
    // https://pest.rs/book/examples/csv.html#writing-the-parser

    let map = parse_as_lines(":0 = ap mod -10");

    use crate::ast::Identifier;
    use crate::ast::Symbol::*;
    let expected = map!(
        Identifier::Var(0) => vec![Ap, Mod, Lit(-10)]
    );

    assert_eq!(expected, map);
    println!("{:?}", map);
}
// 5
// 11
// 21
// 22

/*
ap ap eq x0 x0   =   t
ap ap eq 0 -2   =   f
ap ap eq 0 -1   =   f
ap ap eq 0 0   =   t
ap ap eq 0 1   =   f
ap ap eq 0 2   =   f

t
ap ap t x0 x1   =   x0
ap ap t 1 5   =   1
ap ap t t i   =   t
ap ap t t ap inc 5   =   t
ap ap t ap inc 5 t   =   6
 */
