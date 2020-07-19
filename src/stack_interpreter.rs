use std::collections::HashMap;
use std::ops::Deref;

use crate::ast::lower_symbols;

use super::ast::{Canonicalize, Identifier, Number, Statement, Symbol, SymbolCell};

use crate::ast::modulations;

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

fn op1<F>(env: &StackEnvironment, stack: &mut RuntimeStack, f: F) -> SymbolCell
where
    F: FnOnce(&StackEnvironment, &mut RuntimeStack, SymbolCell) -> SymbolCell,
{
    let op = stack.pop().unwrap();
    f(env, stack, op)
}

fn op2<F>(env: &StackEnvironment, stack: &mut RuntimeStack, f: F) -> SymbolCell
where
    F: FnOnce(&StackEnvironment, &mut RuntimeStack, SymbolCell, SymbolCell) -> SymbolCell,
{
    let op1 = stack.pop().unwrap();
    let op2 = stack.pop().unwrap();
    f(env, stack, op1, op2)
}

fn op3<F>(env: &StackEnvironment, stack: &mut RuntimeStack, f: F) -> SymbolCell
where
    F: FnOnce(
        &StackEnvironment,
        &mut RuntimeStack,
        SymbolCell,
        SymbolCell,
        SymbolCell,
    ) -> SymbolCell,
{
    let op1 = stack.pop().unwrap();
    let op2 = stack.pop().unwrap();
    let op3 = stack.pop().unwrap();
    f(env, stack, op1, op2, op3)
}

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
    fn resolve(symbol: SymbolCell, env: &StackEnvironment, stack: &mut RuntimeStack) -> SymbolCell {
        run_expression(symbol, env, stack);
        stack.pop().unwrap()
    }

    match function.deref() {
        Symbol::Inc => stack_lit1(environment, stack, |x| x + 1),
        Symbol::Dec => stack_lit1(environment, stack, |x| x - 1),
        Symbol::Add => stack_lit2(environment, stack, |x, y| x + y),
        Symbol::Mul => stack_lit2(environment, stack, |x, y| x * y),
        Symbol::Div => stack_lit2(environment, stack, |x, y| x / y),
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
        Symbol::Eq => {
            let first = {
                let pop = stack.pop().unwrap();
                resolve(pop, environment, stack)
            };

            let second = {
                let pop = stack.pop().unwrap();
                resolve(pop, environment, stack)
            };

            if first.deref() == second.deref() {
                Symbol::T.into()
            } else {
                Symbol::F.into()
            }
        }

        Symbol::Lt => stack_lit2(environment, stack, |x, y| {
            if x < y {
                Symbol::T
            } else {
                Symbol::F
            }
        }),

        Symbol::T => {
            let first = stack.pop().unwrap();
            let _ = stack.pop().unwrap();

            resolve(first, environment, stack)
        }
        Symbol::F => {
            let _ = stack.pop().unwrap();
            let second = stack.pop().unwrap();

            resolve(second, environment, stack)
        }
        Symbol::Mod => op1(environment, stack, |env, stack, op| {
            Symbol::Modulated(modulations::modulate(resolve(op, env, stack).deref())).into()
        }),
        Symbol::Dem => op1(environment, stack, |env, stack, op| {
            match resolve(op, env, stack).deref() {
                Symbol::Modulated(val) => modulations::demodulate(val.clone()).into(),
                _ => unreachable!("Dem with invalid operands"),
            }
        }),
        // Symbol::Send => {},
        Symbol::Neg => stack_lit1(environment, stack, |x| Symbol::Lit(-x.clone())),

        // Symbol::Ap => match operands.split_first() {
        //     Some((hd, tl)) => {
        //         dbg!(&tl);
        //         let mut tl = tl.iter().map(|x| x.eval(vars).into()).collect();
        //         eval_thunks(&apply(hd.clone(), vec![], vars), &mut tl, vars)
        //     }
        //     None => unreachable!(),
        // },
        //
        Symbol::Pwr2 => stack_lit1(environment, stack, |x| i64::pow(2, x as u32)),
        Symbol::I => op1(environment, stack, |env, stack, op| op.clone()),

        Symbol::Cons => op2(environment, stack, |env, stack, op1, op2| {
            Symbol::Pair(op1.clone(), op2.clone()).into()
        }),
        Symbol::Car => op1(environment, stack, |env, stack, op| {
            match resolve(op, env, stack).deref() {
                Symbol::List(_) => unreachable!("List should have been lowered"),
                op => unreachable!("Car with invalid operands: {:?}", op),
            }
        }),
        Symbol::Cdr => op1(environment, stack, |env, stack, op| {
            match resolve(op, env, stack).deref() {
                Symbol::Pair(_, v2) => v2.clone(),
                Symbol::List(_) => unreachable!("List should have been lowered"),
                op => unreachable!("Cdr with invalid operands: {:?}", op),
            }
        }),
        Symbol::Nil => Symbol::Nil.into(),
        Symbol::IsNil => op1(environment, stack, |env, stack, op| {
            let op = resolve(op, env, stack);
            if op.deref() == &Symbol::Nil {
                Symbol::T.into()
            } else {
                Symbol::F.into()
            }
        }),

        // Symbol::Draw => {},
        Symbol::Checkerboard => stack_lit2(environment, stack, |x, y| {
            let x_axis = (0..=x).step_by(2);
            let y_axis = (0..=y).step_by(2);
            Symbol::List(
                x_axis
                    .flat_map(|x| {
                        y_axis.clone().map(move |y| {
                            Symbol::Pair(Symbol::Lit(x).into(), Symbol::Lit(y).into())
                        })
                    })
                    .collect::<Vec<_>>(),
            )
            .canonicalize()
        }),
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
