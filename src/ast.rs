// https://message-from-space.readthedocs.io/en/latest/message7.html

use std::collections::{HashMap, VecDeque};
use std::ops::Deref;
use std::rc::Rc;

pub use modulations::{demodulate_string, modulate_to_string};

type Number = i64;
type SymbolCell = Rc<Symbol>;
type Environment = HashMap<Identifier, SymbolCell>;

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
    PartFn(SymbolCell, Vec<SymbolCell>, i8),
    Pair(SymbolCell, SymbolCell),
    Modulated(modulations::Modulated),
}

impl Symbol {
    pub fn canonicalize(&self) -> Self {
        match self {
            Symbol::List(v) => v.iter().rfold(Symbol::Nil, |acc, v| {
                Symbol::Pair(v.clone().into(), acc.into())
            }),
            a => a.clone(),
        }
    }

    fn eval(&self, env: &Environment) -> Symbol {
        eval_thunks(&SymbolCell::new(self.clone()), &mut vec![], env)
            .deref()
            .clone()
    }

    pub fn force(&self, env: &Environment) -> Symbol {
        match self {
            Symbol::List(xs) => Symbol::List(xs.iter().map(|x| x.force(env)).collect()),
            pfn @ Symbol::PartFn(_, _, _) => pfn.eval(env),
            Symbol::Pair(hd, tl) => {
                Symbol::Pair(hd.deref().force(env).into(), tl.deref().force(env).into())
            }
            _ => self.clone(),
        }
    }
}

pub fn eval_instructions<T: Into<SymbolCell> + Clone>(symbols: &[T]) -> Symbol {
    let mut vars = Environment::new();

    let instructions: Vec<SymbolCell> = symbols.iter().map(|sym| sym.clone().into()).collect();

    eval(&instructions, &vars).deref().clone().eval(&vars)
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

fn op1<F>(operands: &[SymbolCell], f: F) -> SymbolCell
where
    F: FnOnce(&Symbol) -> SymbolCell,
{
    let op = operands[0].deref();
    f(op)
}

fn op2<F>(operands: &[SymbolCell], f: F) -> SymbolCell
where
    F: FnOnce(&Symbol, &Symbol) -> SymbolCell,
{
    let op = operands[0].deref();
    let op2 = operands[1].deref();
    f(op, op2)
}

fn op3<F>(operands: &[SymbolCell], f: F) -> SymbolCell
where
    F: FnOnce(&Symbol, &Symbol, &Symbol) -> SymbolCell,
{
    let op = operands[0].deref();
    let op2 = operands[1].deref();
    let op3 = operands[2].deref();
    f(op, op2, op3)
}

impl Into<Symbol> for Number {
    fn into(self) -> Symbol {
        Symbol::Lit(self)
    }
}

fn lit1<T: Into<Symbol>>(
    operands: Vec<SymbolCell>,
    env: &Environment,
    f: fn(Number) -> T,
) -> SymbolCell {
    op1(&operands, move |symbol| match symbol.eval(env) {
        Symbol::Lit(x) => f(x).into().into(),
        _ => unreachable!("Non-literal operand: {:?}", symbol),
    })
}

fn lit2<T: Into<Symbol>>(
    operands: Vec<SymbolCell>,
    env: &Environment,
    f: fn(Number, Number) -> T,
) -> SymbolCell {
    op2(&operands, |op1, op2| match (op1.eval(env), op2.eval(env)) {
        (Symbol::Lit(x), Symbol::Lit(y)) => f(x, y).into().into(),
        _ => unreachable!("Non-literal operands: {:?}", (op1, op2)),
    })
}

fn eval_thunks(op: &SymbolCell, operands: &mut Vec<SymbolCell>, vars: &Environment) -> SymbolCell {
    dbg!(&op);

    match op.deref() {
        Symbol::S => {
            // https://en.wikipedia.org/wiki/SKI_combinator_calculus
            // Sxyz = xz(yz)

            if let [x, y, z] = &operands[0..3] {
                let s = Symbol::PartFn(
                    Symbol::Ap.into(),
                    vec![
                        Symbol::PartFn(
                            Symbol::Ap.into(),
                            vec![x.clone(), z.clone()],
                            num_args(x) - 1,
                        )
                        .into(),
                        Symbol::PartFn(
                            Symbol::Ap.into(),
                            vec![y.clone(), z.clone()],
                            num_args(y) - 1,
                        )
                        .into(),
                    ],
                    0,
                )
                .into();
                dbg!(&s);

                s
            } else {
                unreachable!("{:?}", operands)
            }
        }

        Symbol::C => {
            // https://en.wikipedia.org/wiki/B,_C,_K,_W_system
            // C x y z = x z y

            if let [x, y, z] = operands.as_slice() {
                Symbol::PartFn(
                    Symbol::Ap.into(),
                    vec![
                        Symbol::PartFn(Symbol::Ap.into(), vec![x.clone(), z.clone()], 0).into(),
                        y.clone(),
                    ],
                    0,
                )
                .into()
            } else {
                unreachable!()
            }
        }

        Symbol::B => {
            // https://en.wikipedia.org/wiki/B,_C,_K,_W_system
            // B x y z = x (y z)

            if let [x, y, z] = operands.as_slice() {
                Symbol::PartFn(
                    Symbol::Ap.into(),
                    vec![
                        x.clone(),
                        Symbol::PartFn(Symbol::Ap.into(), vec![y.clone(), z.clone()], 0).into(),
                    ],
                    0,
                )
                .into()
            } else {
                unreachable!()
            }
        }

        Symbol::PartFn(op, args, 0) => match args.split_first() {
            Some((hd, tl)) => {
                assert_eq!(op.deref(), &Symbol::Ap);
                eval_thunks(&hd, &mut tl.to_vec(), vars)
            }
            _ => unreachable!(),
        },

        Symbol::PartFn(op, args, remaining) => {
            assert!(*remaining > 0);
            let mut vec = args.clone();
            vec.push(operands.pop().unwrap());
            Symbol::PartFn(op.clone(), vec, remaining - 1).into()
        }

        Symbol::Ap => {
            let arg = eval_thunks(&operands.pop().unwrap(), &mut vec![], vars);
            apply(arg, operands.to_vec(), vars)
        }

        _ => op.clone(),
    }
}

fn apply(op: SymbolCell, operands: Vec<SymbolCell>, vars: &Environment) -> SymbolCell {
    dbg!(&op);

    match op.deref() {
        Symbol::Lit(_) => op,
        Symbol::Eq => {
            if let [lhs, rhs] = operands.as_slice() {
                if lhs.eval(vars) == rhs.eval(vars) {
                    Symbol::T.into()
                } else {
                    Symbol::F.into()
                }
            } else {
                unreachable!("{:?}", operands)
            }
        }
        Symbol::Inc => lit1(operands, vars, |x| x + 1),

        Symbol::Dec => lit1(operands, vars, |x| x - 1),

        Symbol::Add => lit2(operands, vars, |x, y| x + y),

        Symbol::Var(idx) => vars[&Identifier::Var(*idx)].clone(),

        Symbol::Mul => lit2(operands, vars, |x, y| x * y),

        Symbol::Div => lit2(operands, vars, |x, y| x / y),

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
        Symbol::Lt => lit2(
            operands,
            vars,
            |x, y| {
                if x < y {
                    Symbol::T
                } else {
                    Symbol::F
                }
            },
        ),
        Symbol::Mod => op1(&operands, |op| {
            Symbol::Modulated(modulations::modulate(&op.deref().eval(vars))).into()
        }),
        Symbol::Dem => op1(&operands, |op| match op.eval(vars) {
            Symbol::Modulated(val) => modulations::demodulate(val.clone()).into(),
            _ => unreachable!("Dem with invalid operands"),
        }),
        // Symbol::Send => {},
        Symbol::Neg => lit1(operands, vars, |x| Symbol::Lit(-x.clone())),

        // Symbol::Ap => match operands.split_first() {
        //     Some((hd, tl)) => {
        //         dbg!(&tl);
        //         let mut tl = tl.iter().map(|x| x.eval(vars).into()).collect();
        //         eval_thunks(&apply(hd.clone(), vec![], vars), &mut tl, vars)
        //     }
        //     None => unreachable!(),
        // },
        //
        Symbol::Pwr2 => lit1(operands, vars, |x| i64::pow(2, x as u32)),

        Symbol::I => {
            if let [x] = operands.as_slice() {
                x.clone()
            } else {
                unreachable!()
            }
        }

        Symbol::Cons => {
            if let [x, y] = operands.as_slice() {
                Symbol::Pair(x.clone(), y.clone()).into()
            } else {
                unreachable!()
            }
        }
        Symbol::Car => op1(&operands, |op| match op.eval(vars) {
            Symbol::Pair(v1, _) => v1.clone(),
            Symbol::List(_) => unreachable!("List should have been lowered"),
            _ => unreachable!("Mod with invalid operands"),
        }),

        Symbol::Cdr => op1(&operands, |op| match op.eval(vars) {
            Symbol::Pair(_, v2) => v2.clone(),
            Symbol::List(_) => unreachable!("List should have been lowered"),
            _ => unreachable!("Mod with invalid operands"),
        }),

        Symbol::Nil => Symbol::Nil.into(),

        Symbol::IsNil => op1(&operands, |op| {
            if op.eval(vars) == Symbol::Nil {
                Symbol::T.into()
            } else {
                Symbol::F.into()
            }
        }),

        Symbol::List(_) => unreachable!("List should have been lowered"),

        // Symbol::Draw => {},
        Symbol::Checkerboard => lit2(operands, vars, |x, y| {
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
        // Symbol::MultipleDraw => {},
        Symbol::If0 => {
            if let [literal, x, y] = operands.as_slice() {
                if literal.deref().eval(vars) == Symbol::Lit(0) {
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
        // pfn @ Symbol::PartFn(_, _, _) => {
        //     let mut args = operands.clone();
        //     eval_thunks(&op, &mut args, vars)
        // }
        //
        _ => unimplemented!("{0:?} is not implemented", op),
    }
}

pub fn interpret(statements: Vec<Statement>) -> SymbolCell {
    #[derive(Clone, Debug)]
    struct Result {
        environment: Environment,
        last: Option<SymbolCell>,
    }

    impl Default for Result {
        fn default() -> Self {
            Self {
                environment: Environment::new(),
                last: None,
            }
        }
    }

    statements
        .iter()
        .fold(Result::default(), |mut acc, statement| {
            let symbol = eval(&statement.1, &acc.environment);
            acc.environment.insert(statement.0.clone(), symbol.clone());
            Result {
                last: Some(symbol),
                ..acc
            }
        })
        .last
        .unwrap()
}

fn lower_symbols<T>(symbols: &[T]) -> Vec<SymbolCell>
where
    T: Into<SymbolCell> + Clone,
{
    symbols
        .iter()
        .map(|inst| inst.clone().into().canonicalize().into())
        .collect()
}

pub fn eval<T>(instructions: &[T], vars: &Environment) -> SymbolCell
where
    T: Into<SymbolCell> + Clone,
{
    let mut stack = Vec::<SymbolCell>::new();

    let lowered_symbols: Vec<SymbolCell> = lower_symbols(instructions);

    for inst in lowered_symbols.iter().rev() {
        stack.reverse();
        let val = eval_thunks(inst, &mut stack, vars);
        stack.reverse();
        stack.push(val);
    }

    dbg!(&stack);

    // assert_eq!(stack.len(), 1);

    stack.pop().unwrap().force(vars).into()
}

#[cfg(test)]
mod tests;
