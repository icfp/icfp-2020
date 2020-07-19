use std::collections::HashMap;
use std::ops::Deref;

use crate::ast::lower_symbols;

use super::ast::{Identifier, Number, Statement, Symbol, SymbolCell};

type StackEnvironment = HashMap<Identifier, SymbolCell>;

pub fn build_symbol_tree<T>(instructions: &[T]) -> SymbolCell
where
    T: Into<SymbolCell> + Clone,
{
    let mut stack = Vec::<SymbolCell>::new();

    let lowered_symbols: Vec<SymbolCell> = lower_symbols(instructions);

    for inst in lowered_symbols.iter().rev() {
        let val = lower_applies(inst, &mut stack);
        stack.push(val);
    }

    // dbg!(&stack);

    assert_eq!(stack.len(), 1);

    let last: SymbolCell = stack.pop().unwrap().into();
    assert!(match last.deref() {
        Symbol::ApplyPair(_, _) | Symbol::Var(_) | Symbol::Lit(_) => true,
        _ => false,
    });

    last
}

fn lower_applies(op: &SymbolCell, operands: &mut Vec<SymbolCell>) -> SymbolCell {
    match op.deref() {
        Symbol::Ap => {
            let fun = operands.pop().unwrap();
            let arg = operands.pop().unwrap();
            Symbol::ApplyPair(fun, arg).into()
        }
        _ => op.clone(),
    }
}

type RuntimeStack = Vec<SymbolCell>;

fn stack_lit1<T: Into<Symbol>>(
    env: &StackEnvironment,
    stack: &mut RuntimeStack,
    f: fn(Number) -> T,
) -> SymbolCell {
    let arg = {
        let pop = stack.pop().unwrap();
        run_expression(pop, env, stack);
        stack.pop().unwrap()
    };

    match arg.deref() {
        Symbol::Lit(x) => f(*x).into().into(),
        _ => unreachable!("Non-literal operand: {:?}", arg),
    }
}

fn stack_lit2<T: Into<Symbol>>(
    env: &StackEnvironment,
    stack: &mut RuntimeStack,
    f: fn(Number, Number) -> T,
) -> SymbolCell {
    let first = {
        let pop = stack.pop().unwrap();
        run_expression(pop, env, stack);
        stack.pop().unwrap()
    };

    let second = {
        let pop = stack.pop().unwrap();
        run_expression(pop, env, stack);
        stack.pop().unwrap()
    };

    match (first.deref(), second.deref()) {
        (Symbol::Lit(x), Symbol::Lit(y)) => f(*x, *y).into().into(),
        _ => unreachable!("Non-literal operands: {:?}", (first, second)),
    }
}

pub fn run_function(
    function: SymbolCell,
    environment: &StackEnvironment,
    stack: &mut RuntimeStack,
) -> SymbolCell {
    match function.deref() {
        Symbol::Inc => stack_lit1(environment, stack, |x| x + 1),
        Symbol::Dec => stack_lit1(environment, stack, |x| x - 1),
        Symbol::Add => stack_lit2(environment, stack, |x, y| x + y),
        Symbol::Mul => stack_lit2(environment, stack, |x, y| x * y),
        Symbol::If0 => {
            let test = stack.pop().unwrap();
            let first = stack.pop().unwrap();
            let second = stack.pop().unwrap();
            if test.deref() == &Symbol::Lit(0) {
                run_expression(first, environment, stack);
            } else {
                run_expression(second, environment, stack);
            }
            stack.pop().unwrap()
        }
        Symbol::S => {
            // https://en.wikipedia.org/wiki/SKI_combinator_calculus
            // Sxyz = xz(yz)
            let x = stack.pop().unwrap();
            let y = stack.pop().unwrap();
            let z = stack.pop().unwrap();

            let fn0 = Symbol::ApplyPair(x.clone(), z.clone());
            let fn1 = Symbol::ApplyPair(y.clone(), z.clone());
            let s = Symbol::ApplyPair(fn0.into(), fn1.into()).into();
            // dbg!(&s);

            run_expression(s, environment, stack);
            stack.pop().unwrap()
        }
        Symbol::C => {
            // https://en.wikipedia.org/wiki/B,_C,_K,_W_system
            // C x y z = x z y

            let x = stack.pop().unwrap();
            let y = stack.pop().unwrap();
            let z = stack.pop().unwrap();

            let xz_apply = Symbol::ApplyPair(x.clone(), z.clone());
            let c = Symbol::ApplyPair(xz_apply.into(), y.clone()).into();
            run_expression(c, environment, stack);
            stack.pop().unwrap()
        }

        Symbol::B => {
            // https://en.wikipedia.org/wiki/B,_C,_K,_W_system
            // B x y z = x (y z)

            let x = stack.pop().unwrap();
            let y = stack.pop().unwrap();
            let z = stack.pop().unwrap();

            let b =
                Symbol::ApplyPair(x.clone(), Symbol::ApplyPair(y.clone(), z.clone()).into()).into();

            run_expression(b, environment, stack);
            stack.pop().unwrap()
        }
        func => unimplemented!("Function not supported: {:?}", func),
    }
}

pub fn run_expression(
    symbol: SymbolCell,
    environment: &StackEnvironment,
    stack: &mut RuntimeStack,
) {
    match symbol.deref() {
        Symbol::ApplyPair(fun, arg) => {
            stack.push(arg.clone());

            let result = match fun.deref() {
                Symbol::ApplyPair(_, _) | Symbol::Var(_) => {
                    run_expression(fun.clone(), environment, stack);
                    stack.pop().unwrap()
                }
                _ => run_function(fun.clone(), environment, stack),
            };

            stack.push(result);
        }
        Symbol::Var(id) => run_expression(
            environment[&Identifier::Var(*id)].clone(),
            environment,
            stack,
        ),
        Symbol::Lit(_) => stack.push(symbol.clone()),
        op => unreachable!("Operand: {:?}", op),
    }
}

pub fn run(symbol: SymbolCell, environment: &StackEnvironment) -> SymbolCell {
    let mut stack = RuntimeStack::new();
    run_expression(symbol, environment, &mut stack);
    stack.pop().unwrap().clone()
}

pub fn stack_interpret(statements: Vec<Statement>) -> Symbol {
    let mut env = StackEnvironment::new();
    let last_statement_id = statements.last().unwrap().0.clone();

    for statement in statements.clone() {
        let statements_rvalue = dbg!(build_symbol_tree(&statement.1));

        env.insert(statement.0, statements_rvalue);
    }

    let last_rvalue = env[&last_statement_id].clone();

    let result = run(last_rvalue, &env);

    result.deref().clone()
}

#[cfg(test)]
mod stack_tests {
    use crate::parser::parse_as_lines;

    use super::stack_interpret;
    use super::Symbol::*;

    #[test]
    fn inc() {
        let lines = parse_as_lines(":1 = ap inc 1");
        let _symbol = dbg!(stack_interpret(lines));
    }

    #[test]
    fn add() {
        let lines = parse_as_lines(":1 = ap ap add 2 1");
        let _symbol = dbg!(stack_interpret(lines));
    }

    #[test]
    fn var_literal_lookahead() {
        let lines = parse_as_lines(
            ":1 = ap ap add 2 :2
                                                    :2 = 3
                                                    :3 = :1",
        );
        let _symbol = dbg!(stack_interpret(lines));
    }

    #[test]
    fn var_curry() {
        let lines = parse_as_lines(
            ":1 = ap add 2
                                                    :2 = 3
                                                    :3 = ap :1 :2",
        );
        let _symbol = dbg!(stack_interpret(lines));
    }

    #[test]
    fn var_dead_code() {
        let lines = parse_as_lines(
            ":0 = 5
            :1 = ap ap ap if0 1 :1 :0",
        );
        let _symbol = dbg!(stack_interpret(lines));
    }

    #[test]
    fn s_combinator() {
        let lines = parse_as_lines(":1 = ap ap ap s add inc 1");
        let symbol = dbg!(stack_interpret(lines));
        assert_eq!(symbol, Lit(3));

        let lines = parse_as_lines(":2 = ap ap ap s mul ap add 1 6");
        let symbol = dbg!(stack_interpret(lines));
        assert_eq!(symbol, Lit(42));
    }

    #[test]
    fn c_combinator() {
        let lines = parse_as_lines(":1 = ap ap ap c add 1 2");
        let symbol = dbg!(stack_interpret(lines));
        assert_eq!(symbol, Lit(3));
    }

    #[test]
    fn b_combinator() {
        let lines = parse_as_lines(
            ":0 = 42
        :1 = ap ap ap b inc dec :0",
        );
        let symbol = dbg!(stack_interpret(lines));
        assert_eq!(symbol, Lit(42));
    }

    #[test]
    fn test_lookahead() {
        let lines = parse_as_lines(
            ":1 = ap add 1
             :2 = ap :1 2",
        );

        let symbol = dbg!(stack_interpret(lines));
        assert_eq!(symbol, Lit(3));
    }

    #[test]
    fn run_simple() {
        let lines = parse_as_lines(
            ":1 = ap add 1
:2 = ap ap ap ap if0 1 :2 :1 2
:3 = :2",
        );

        let symbol = dbg!(stack_interpret(lines));
        assert_eq!(symbol, Lit(3));
    }
}
