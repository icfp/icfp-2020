use std::collections::HashMap;
use std::ops::Deref;

use crate::ast::lower_symbols;

use super::ast::{Canonicalize, Identifier, Number, Statement, Symbol, SymbolCell};

use crate::ast::modulations;
use crate::ast::Symbol::ReadyForEval;

type StackEnvironment = HashMap<Identifier, SymbolCell>;

fn build_symbol_tree(statement: &Statement) -> SymbolCell {
    let mut stack = Vec::<SymbolCell>::new();

    let lowered_symbols: Vec<SymbolCell> = lower_symbols(&statement.1);

    for inst in lowered_symbols.iter().rev() {
        let val = lower_applies(inst, &mut stack);
        stack.push(val);
    }

    // dbg!(&stack);

    assert_eq!(
        stack.len(),
        1,
        "Stack contains more than one item: {:?}",
        stack
    );

    let last: SymbolCell = stack.pop().unwrap().into();
    // assert!(
    //     match last.deref() {
    //         Symbol::ApplyPair(_, _) | Symbol::Var(_) | Symbol::Lit(_) | Symbol::T | Symbol::F =>
    //             true,
    //         _ => false,
    //     },
    //     "Statement({:?}) — Unexpected last result: {:?}",
    //     statement,
    //     last
    // );

    last
}

fn lower_applies(op: &SymbolCell, operands: &mut Vec<SymbolCell>) -> SymbolCell {
    match op.deref() {
        Symbol::Ap => {
            let fun = operands.pop().unwrap();
            let arg = operands.pop().unwrap();
            Symbol::Closure {
                captured_arg: arg,
                body: fun,
            }
            .into()
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
        run_expression(pop, env, stack)
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
        run_expression(pop, env, stack)
    };

    let second = {
        let pop = stack.pop().unwrap();
        run_expression(pop, env, stack)
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
) {
    fn resolve(symbol: SymbolCell, env: &StackEnvironment, stack: &mut RuntimeStack) -> SymbolCell {
        run_expression(symbol, env, stack)
    }

    let result = match function.deref() {
        Symbol::Var(id) => run_expression(
            environment[&Identifier::Var(*id)].clone(),
            environment,
            stack,
        ),
        Symbol::Lit(_) => function.clone(),
        Symbol::T | Symbol::F => function.clone(),
        Symbol::Inc => stack_lit1(environment, stack, |x| x + 1),
        Symbol::Dec => stack_lit1(environment, stack, |x| x - 1),
        Symbol::Add => stack_lit2(environment, stack, |x, y| x + y),
        Symbol::Mul => stack_lit2(environment, stack, |x, y| x * y),
        Symbol::Div => stack_lit2(environment, stack, |x, y| x / y),
        Symbol::If0 => op3(environment, stack, |env, stack, test, first, second| {
            if resolve(test, env, stack).deref() == &Symbol::Lit(0) {
                resolve(first, env, stack)
            } else {
                resolve(second, env, stack)
            }
        }),
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

        Symbol::Pwr2 => stack_lit1(environment, stack, |x| i64::pow(2, x as u32)),
        Symbol::I => op1(environment, stack, |_, _, op| op.clone()),

        Symbol::Cons => op2(environment, stack, |_, _, op1, op2| {
            Symbol::Pair(op1.clone(), op2.clone()).into()
        }),
        Symbol::Car => op1(environment, stack, |env, stack, op| {
            match resolve(op, env, stack).deref() {
                Symbol::Pair(hd, _) => resolve(hd.clone(), env, stack),
                Symbol::List(_) => unreachable!("List should have been lowered"),
                op => unreachable!("Car with invalid operands: {:?}", op),
            }
        }),
        Symbol::Cdr => op1(environment, stack, |env, stack, op| {
            match resolve(op, env, stack).deref() {
                // this one should be lazy and not resolve, i think...
                Symbol::Pair(_, tail) => tail.clone(),
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

            let fn0 = Symbol::ReadyForEval(x.clone(), z.clone());
            let fn1 = Symbol::ReadyForEval(y.clone(), z.clone());
            let s = Symbol::ReadyForEval(fn0.into(), fn1.into()).into();
            // dbg!(&s);

            run_expression(s, environment, stack);
            stack.pop().unwrap()
        }
        Symbol::Closure {
            captured_arg: arg,
            body: fun,
        } => {
            stack.push(arg.clone());
            fun.clone()
        }
        Symbol::C => {
            // https://en.wikipedia.org/wiki/B,_C,_K,_W_system
            // C x y z = x z y

            let x = stack.pop().unwrap();
            let y = stack.pop().unwrap();
            let z = stack.pop().unwrap();

            let xz_apply = Symbol::Closure {
                captured_arg: z,
                body: x,
            };

            let c = Symbol::Closure {
                captured_arg: y,
                body: xz_apply.into(),
            }
            .into();
            c
        }

        Symbol::B => {
            // https://en.wikipedia.org/wiki/B,_C,_K,_W_system
            // B x y z = x (y z)

            let x = stack.pop().unwrap();
            let y = stack.pop().unwrap();
            let z = stack.pop().unwrap();

            let b =
                Symbol::ReadyForEval(x.clone(), Symbol::ReadyForEval(y.clone(), z.clone()).into())
                    .into();

            run_expression(b, environment, stack);
            stack.pop().unwrap()
        }
        func => unimplemented!("Function not supported: {:?}", func),
    };

    stack.push(dbg!(result));
}

pub fn run_expression(
    symbol: SymbolCell,
    environment: &StackEnvironment,
    stack: &mut RuntimeStack,
) -> SymbolCell {
    let mut op: SymbolCell = symbol;
    loop {
        dbg!(&op);
        dbg!(&stack);
        run_function(op.clone(), environment, stack);
        let sym: SymbolCell = stack.pop().unwrap();
        match sym.deref() {
            ops if ops.num_args() > 0 => op = sym.clone(),
            op => return op.into(),
        }
    }
}

pub fn run(symbol: SymbolCell, environment: &StackEnvironment) -> SymbolCell {
    let mut stack = RuntimeStack::new();
    run_expression(symbol, environment, &mut stack)
}

pub fn stack_interpret(statements: Vec<Statement>) -> Symbol {
    let mut env = StackEnvironment::new();
    let last_statement_id = statements.last().unwrap().0.clone();

    for statement in statements.clone() {
        let statements_rvalue = dbg!(build_symbol_tree(&statement));

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
    use crate::ast::Symbol;

    fn run_test(program: &str, expectation: Symbol) {
        let lines = parse_as_lines(program);
        let symbol = dbg!(stack_interpret(lines));
        assert_eq!(symbol, expectation);
    }

    #[test]
    fn inc() {
        let lines = parse_as_lines(":1 = ap inc 1");
        let _symbol = dbg!(stack_interpret(lines));
    }

    #[test]
    fn add() {
        let lines = parse_as_lines(":1 = ap ap add 2 1");
        let symbol = dbg!(stack_interpret(lines));
        assert_eq!(symbol, Lit(3))
    }

    #[test]
    fn mul() {
        // https://message-from-space.readthedocs.io/en/latest/message9.html

        /*
        ap ap mul 4 2   =   8
        ap ap mul 3 4   =   12
        ap ap mul 3 -2   =   -6
        ap ap mul x0 x1   =   ap ap mul x1 x0
        ap ap mul x0 0   =   0
        ap ap mul x0 1   =   x0
        */
        run_test(":1 = ap ap mul 4 2", Lit(8));
        run_test(":1 = ap ap mul 3 4", Lit(12));
        run_test(":1 = ap ap mul 3 -2", Lit(-6));
        run_test(":1 = ap ap mul -2 3", Lit(-6));
        run_test(":1 = ap ap mul 4 0", Lit(0));
        run_test(":1 = ap ap mul 4 1", Lit(4));
    }
    #[test]
    fn less_than() {
        // message 12
        /*
        ap ap lt 0 -1   =   f
        ap ap lt 0 0   =   f
        ap ap lt 0 1   =   t
        ap ap lt 0 2   =   t
        ...
        ap ap lt 1 0   =   f
        ap ap lt 1 1   =   f
        ap ap lt 1 2   =   t
        ap ap lt 1 3   =   t
        ...
        ap ap lt 2 1   =   f
        ap ap lt 2 2   =   f
        ap ap lt 2 3   =   t
        ap ap lt 2 4   =   t
        ...
        ap ap lt 19 20   =   t
        ap ap lt 20 20   =   f
        ap ap lt 21 20   =   f
        ...
        ap ap lt -19 -20   =   f
        ap ap lt -20 -20   =   f
        ap ap lt -21 -20   =   t
        */
        run_test(":1 = ap ap lt 0 -1", F);
        run_test(":1 = ap ap lt 0 0", F);
        run_test(":1 = ap ap lt 0 1", T);
    }

    #[test]
    fn div() {
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

        run_test(":1 = ap ap div 4 2", Lit(2));
        run_test(":1 = ap ap div 4 3", Lit(1));
        run_test(":1 = ap ap div 4 5", Lit(0));
        run_test(":1 = ap ap div 5 2", Lit(2));
        run_test(":1 = ap ap div 6 -2", Lit(-3));
        run_test(":1 = ap ap div 5 -3", Lit(-1));
        run_test(":1 = ap ap div -5 3", Lit(-1));
        run_test(":1 = ap ap div -5 -3", Lit(1));
    }

    #[test]
    fn equality() {
        run_test(":1 = ap ap eq 1 1", T);
    }

    #[test]
    fn inequality() {
        run_test(":1 = ap ap eq 1 2", F);
    }

    #[test]
    fn cons() {
        run_test(":1 = ap ap cons 1 2", Pair(Lit(1).into(), Lit(2).into()));
    }

    #[test]
    fn car() {
        run_test(":1 = ap car ap ap cons 1 2", Lit(1));
    }

    #[test]
    fn cdr() {
        run_test(":1 = ap cdr ap ap cons 1 2", Lit(2));
    }

    #[test]
    fn true_func() {
        // message 21

        /*
        ap ap t x0 x1   =   x0
        ap ap t 1 5   =   1
        ap ap t t i   =   t
        ap ap t t ap inc 5   =   t
        ap ap t ap inc 5 t   =   6
        */

        run_test(":1 = ap ap t 1 2", Lit(1));
        run_test(":1 = ap ap t t i", T);
        run_test(":1 = ap ap t t ap inc 5", T);
        run_test(":1 = ap ap t ap inc 5 t", Lit(6));
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

    #[test]
    fn galaxy_first_pass() {
        run_test(
            ":1338 = ap ap c ap ap b c ap ap c ap ap b c 1 2 3
             galaxy = :1338",
            Lit(0),
        )
    }
}
