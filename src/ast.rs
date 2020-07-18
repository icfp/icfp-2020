// https://message-from-space.readthedocs.io/en/latest/message7.html

use std::collections::{HashMap, VecDeque};

pub use modulations::{demodulate_string, modulate_to_string};

type Number = i64;

mod modulations;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Statement(pub Identifier, pub Vec<Symbol>);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Identifier {
    Name(String),
    Var(usize),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Symbol {
    Lit(Number),
    // 1-3
    Eq,
    // 4
    Inc,
    // 5
    Dec,
    // 6
    Add,
    // 7
    Var(usize),
    // 8
    Mul,
    // 9
    Div,
    // 10
    T,
    // 11 & 21
    F,
    // 11 & 22
    Lt,
    // 12
    Mod,
    // 13
    Dem,
    // 14
    Send,
    // 15
    Neg,
    // 16
    Ap,
    // 17
    S,
    // 18
    C,
    // 19
    B,
    // 20
    Pwr2,
    // 23
    I,
    // 24
    Cons,
    // 25
    Car,
    // 26
    Cdr,
    // 27
    Nil,
    // 28
    IsNil,
    // 29
    List(Vec<Symbol>),
    // 30
    // 31 .. vec = alias for cons that looks nice in “vector” usage context.
    Draw,
    // 32
    Checkerboard,
    // 33
    MultipleDraw,
    // 34
    // 35 = modulate list, doesn't seem to map to an operation
    // 36 = send 0:
    //   :1678847
    //   ap send ( 0 )   =   ( 1 , :1678847 )
    If0,
    // 37
    Interact,
    // 38
    // 39 = interaction protocol
    StatelessDraw,
    PartFn(Box<Symbol>, Vec<Symbol>, i8),
    Pair(Box<Symbol>, Box<Symbol>),
    Modulated(modulations::Modulated),
}

pub fn eval_instructions(tree: &[Symbol]) -> Symbol {
    let mut vars = HashMap::new();

    eval(tree, &mut vars)
}

fn num_args(symbol: &Symbol) -> i8 {
    match symbol {
        Symbol::Lit(_) => 0,
        Symbol::Eq => 2,
        Symbol::Inc => 1,
        Symbol::Dec => 1,
        Symbol::Add => 2,
        Symbol::Var(_) => 0,
        Symbol::Mul => 2,
        Symbol::Div => 2,
        Symbol::T => 2,
        Symbol::F => 2,
        Symbol::Lt => 2,
        Symbol::Mod => 1,
        Symbol::Dem => 1,
        Symbol::Send => 1,
        Symbol::Neg => 1,
        Symbol::Ap => 2,
        Symbol::S => 3,
        Symbol::C => 3,
        Symbol::B => 3,
        Symbol::Pwr2 => 1,
        Symbol::I => 1,
        Symbol::Cons => 2,
        Symbol::Car => 1,
        Symbol::Cdr => 1,
        Symbol::Nil => 0,
        Symbol::IsNil => 1,
        Symbol::List(_) => 0,
        Symbol::Draw => 1,
        Symbol::Checkerboard => 2,
        Symbol::MultipleDraw => 1,
        Symbol::If0 => 3,
        Symbol::Interact => 3,
        Symbol::StatelessDraw => 3,
        Symbol::PartFn(_, _, i) => *i,
        Symbol::Pair(_, _) => 0,
        Symbol::Modulated(_) => 0,
    }
}

fn lit1(operands: Vec<Symbol>, f: fn(Number) -> Number) -> Symbol {
    if let [Symbol::Lit(x)] = operands.as_slice() {
        Symbol::Lit(f(*x))
    } else {
        unreachable!("{:?}", operands);
    }
}

fn lit2(operands: Vec<Symbol>, f: fn(Number, Number) -> Number) -> Symbol {
    if let [Symbol::Lit(x), Symbol::Lit(y)] = operands.as_slice() {
        Symbol::Lit(f(*x, *y))
    } else {
        unreachable!("{:?}", operands);
    }
}

fn eval_fn(
    op: Symbol,
    operands: &mut VecDeque<Symbol>,
    vars: &mut HashMap<Identifier, Symbol>,
) -> Symbol {
    match num_args(&op) {
        0 => eval_val(op, Vec::new(), vars),
        x if x > 0 => {
            let arg = operands.pop_back().unwrap();
            if let Symbol::PartFn(sym, mut args, _) = op {
                args.push(arg);
                Symbol::PartFn(sym, args, x - 1)
            } else {
                Symbol::PartFn(Box::new(op), vec![arg], x - 1)
            }
        }
        _ => unreachable!(),
    }
}

fn eval_val(
    op: Symbol,
    raw_operands: Vec<Symbol>,
    vars: &mut HashMap<Identifier, Symbol>,
) -> Symbol {
    let operands: Vec<Symbol> = raw_operands
        .iter()
        .map(|x| eval(&[x.clone()], vars))
        .collect();

    match op {
        Symbol::Lit(_) => op,
        Symbol::Eq => {
            if let [lhs, rhs] = operands.as_slice() {
                if lhs == rhs {
                    Symbol::T
                } else {
                    Symbol::F
                }
            } else {
                unreachable!("{:?}", operands)
            }
        }
        Symbol::Inc => lit1(operands, |x| x + 1),

        Symbol::Dec => lit1(operands, |x| x - 1),

        Symbol::Add => lit2(operands, |x, y| x + y),

        Symbol::Var(_) => unreachable!("Should be handled by outer eval loop"),

        Symbol::Mul => lit2(operands, |x, y| x * y),

        Symbol::Div => lit2(operands, |x, y| x / y),

        Symbol::T => {
            if let [t, _] = operands.as_slice() {
                t.clone()
            } else {
                unreachable!("{:?}", operands)
            }
        }
        Symbol::F => {
            if let [_, f] = operands.as_slice() {
                f.clone()
            } else {
                unreachable!("{:?}", operands)
            }
        }
        Symbol::Lt => match operands.as_slice() {
            [Symbol::Lit(x), Symbol::Lit(y)] => {
                if x < y {
                    Symbol::T
                } else {
                    Symbol::F
                }
            }
            _ => unreachable!("Lt with invalid operands"),
        },
        Symbol::Mod => match operands.as_slice() {
            [sym] => Symbol::Modulated(modulations::modulate(sym)),
            _ => unreachable!("Mod with invalid operands"),
        },
        Symbol::Dem => match operands.as_slice() {
            [Symbol::Modulated(val)] => modulations::demodulate(val.clone()),
            _ => unreachable!("Dem with invalid operands"),
        },
        // Symbol::Send => {},
        Symbol::Neg => {
            if let [Symbol::Lit(x)] = operands.as_slice() {
                Symbol::Lit(-x.clone())
            } else {
                unreachable!()
            }
        }
        Symbol::Ap => unreachable!("Should be handled by outer eval loop"),

        Symbol::S => {
            // https://en.wikipedia.org/wiki/SKI_combinator_calculus
            // Sxyz = xz(yz)

            if let [x, y, z] = operands.as_slice() {
                eval(
                    &[
                        Symbol::Ap,
                        Symbol::Ap,
                        x.clone(),
                        z.clone(),
                        Symbol::Ap,
                        y.clone(),
                        z.clone(),
                    ],
                    vars,
                )
            } else {
                unreachable!()
            }
        }

        Symbol::C => {
            // https://en.wikipedia.org/wiki/B,_C,_K,_W_system
            // C x y z = x z y

            if let [x, y, z] = operands.as_slice() {
                eval(
                    &[Symbol::Ap, Symbol::Ap, x.clone(), z.clone(), y.clone()],
                    vars,
                )
            } else {
                unreachable!()
            }
        }

        Symbol::B => {
            // https://en.wikipedia.org/wiki/B,_C,_K,_W_system
            // B x y z = x (y z)

            if let [x, y, z] = operands.as_slice() {
                eval(
                    &[Symbol::Ap, x.clone(), Symbol::Ap, y.clone(), z.clone()],
                    vars,
                )
            } else {
                unreachable!()
            }
        }

        Symbol::Pwr2 => lit1(operands, |x| i64::pow(2, x as u32)),

        Symbol::I => {
            if let [x] = operands.as_slice() {
                x.clone()
            } else {
                unreachable!()
            }
        }

        Symbol::Cons => {
            if let [x, y] = operands.as_slice() {
                Symbol::Pair(Box::from(x.clone()), Box::from(y.clone()))
            } else {
                unreachable!()
            }
        }
        Symbol::Car => match operands.as_slice() {
            [Symbol::Pair(v1, _)] => *(v1.clone()),
            [Symbol::List(v)] => v.first().unwrap().clone(),
            _ => unreachable!("Mod with invalid operands"),
        },

        Symbol::Cdr => match operands.as_slice() {
            [Symbol::Pair(_, v2)] => *(v2.clone()),
            [Symbol::List(v)] => Symbol::List(v.iter().cloned().skip(1).collect()),
            _ => unreachable!("Mod with invalid operands"),
        },

        Symbol::Nil => Symbol::Nil,

        Symbol::IsNil => {
            if let [x] = operands.as_slice() {
                if x == &Symbol::Nil {
                    Symbol::T
                } else {
                    Symbol::F
                }
            } else {
                unreachable!()
            }
        }

        Symbol::List(v) => v.iter().rfold(Symbol::Nil, |acc, v| {
            Symbol::Pair(Box::from(v.clone()), Box::from(acc))
        }),

        // Symbol::Draw => {},
        Symbol::Checkerboard => {
            if let [Symbol::Lit(x), Symbol::Lit(y)] = operands.as_slice() {
                let x_axis = (0..=*x).step_by(2);
                let y_axis = (0..=*y).step_by(2);

                return Symbol::List(
                    x_axis
                        .flat_map(|x| {
                            y_axis.clone().map(move |y| {
                                Symbol::Pair(Box::from(Symbol::Lit(x)), Box::from(Symbol::Lit(y)))
                            })
                        })
                        .collect::<Vec<_>>(),
                );
            } else {
                unreachable!()
            }
        }
        // Symbol::MultipleDraw => {},
        Symbol::If0 => {
            if let [literal, x, y] = operands.as_slice() {
                if literal == &Symbol::Lit(0) {
                    x.clone()
                } else {
                    y.clone()
                }
            } else {
                unreachable!("{:?}", operands)
            }
        }
        // Symbol::Interact => {},
        // Symbol::StatelessDraw => {},
        Symbol::PartFn(_op0, _args, 0) => unreachable!("Should be handled by outer eval loop"),

        _ => unimplemented!("{0:?} is not implemented", op),
    }
}

fn eval_thunk(instruction: Symbol, vars: &mut HashMap<Identifier, Symbol>) -> Symbol {
    match instruction {
        Symbol::PartFn(op, operands, 0) => eval_val(*op.clone(), operands.clone(), vars),

        _ => instruction,
    }
}

pub fn interpret(statements: Vec<Statement>) -> Symbol {
    #[derive(Clone, Debug)]
    struct Result {
        environment: HashMap<Identifier, Symbol>,
        last: Option<Symbol>,
    }

    impl Default for Result {
        fn default() -> Self {
            Self {
                environment: HashMap::new(),
                last: None,
            }
        }
    }

    statements
        .iter()
        .fold(Result::default(), |mut acc, statement| {
            let symbol = eval(&statement.1, &mut acc.environment);
            acc.environment.insert(statement.0.clone(), symbol.clone());
            Result {
                last: Some(symbol),
                ..acc
            }
        })
        .last
        .unwrap()
}

pub fn eval(instructions: &[Symbol], vars: &mut HashMap<Identifier, Symbol>) -> Symbol {
    let mut stack = VecDeque::<Symbol>::new();
    for inst in instructions.iter().rev() {
        match inst {
            Symbol::Ap => {
                let op = stack.pop_back().unwrap();
                let res = eval_fn(op, &mut stack, vars);
                stack.push_back(res);
            }

            Symbol::Var(idx) => stack.push_back(
                vars.get(&Identifier::Var(*idx))
                    .expect(&format!("Unable to find variable {}", idx))
                    .clone(),
            ),

            _ => stack.push_back(eval_thunk(inst.clone(), vars)),
        }
    }

    assert_eq!(stack.len(), 1);

    eval_thunk(stack.pop_back().unwrap(), vars)
}

#[cfg(test)]
mod tests;
