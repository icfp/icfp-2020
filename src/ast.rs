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
}

pub fn eval_instructions<T: Into<SymbolCell> + Clone>(symbols: &[T]) -> Symbol {
    let mut vars = Environment::new();

    let instructions: Vec<SymbolCell> = symbols.iter().map(|sym| sym.clone().into()).collect();

    eval(&instructions, &vars).deref().clone()
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

fn lit1<T: Into<Symbol>>(operands: Vec<SymbolCell>, f: fn(Number) -> T) -> SymbolCell {
    op1(&operands, move |symbol| match symbol {
        Symbol::Lit(x) => f(*x).into().into(),
        _ => unreachable!("Non-literal operand: {:?}", symbol),
    })
}

fn lit2<T: Into<Symbol>>(operands: Vec<SymbolCell>, f: fn(Number, Number) -> T) -> SymbolCell {
    op2(&operands, |op1, op2| match (op1, op2) {
        (Symbol::Lit(x), Symbol::Lit(y)) => f(*x, *y).into().into(),
        _ => unreachable!("Non-literal operands: {:?}", (op1, op2)),
    })
}

fn eval_fn(op: SymbolCell, operands: &mut VecDeque<SymbolCell>, vars: &Environment) -> SymbolCell {
    match num_args(&op) {
        0 => eval_val(op, Vec::new(), vars),
        x if x > 0 => {
            let arg = operands.pop_back().unwrap();
            if let Symbol::PartFn(sym, args, _) = op.deref() {
                let mut new_args = args.clone();
                new_args.push(arg);
                Symbol::PartFn(Rc::clone(sym), new_args, x - 1).into()
            } else {
                Symbol::PartFn(op.into(), vec![arg], x - 1).into()
            }
        }
        _ => unreachable!(),
    }
}

fn eval_val(op: SymbolCell, raw_operands: Vec<SymbolCell>, vars: &Environment) -> SymbolCell {
    let operands: Vec<SymbolCell> = raw_operands
        .iter()
        .map(|x| eval(&[SymbolCell::clone(x)], vars))
        .collect();

    match op.deref() {
        Symbol::Lit(_) => op,
        Symbol::Eq => {
            if let [lhs, rhs] = operands.as_slice() {
                if lhs == rhs {
                    Symbol::T.into()
                } else {
                    Symbol::F.into()
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
        Symbol::Lt => op2(&operands, |op1, op2| match (op1, op2) {
            (&Symbol::Lit(x), &Symbol::Lit(y)) => {
                if x < y {
                    Symbol::T.into()
                } else {
                    Symbol::F.into()
                }
            }
            _ => unreachable!("Lt with invalid operands"),
        }),
        Symbol::Mod => op1(&operands, |op| match op {
            sym => Symbol::Modulated(modulations::modulate(sym.deref())).into(),
            _ => unreachable!("Mod with invalid operands"),
        }),
        Symbol::Dem => op1(&operands, |op| match op {
            Symbol::Modulated(val) => modulations::demodulate(val.clone()).into(),
            _ => unreachable!("Dem with invalid operands"),
        }),
        // Symbol::Send => {},
        Symbol::Neg => op1(&operands, |op| {
            if let Symbol::Lit(x) = op {
                Symbol::Lit(-x.clone()).into()
            } else {
                unreachable!()
            }
        }),
        Symbol::Ap => unreachable!("Should be handled by outer eval loop"),

        Symbol::S => {
            // https://en.wikipedia.org/wiki/SKI_combinator_calculus
            // Sxyz = xz(yz)

            if let [x, y, z] = operands.as_slice() {
                eval(
                    &[
                        Symbol::Ap.into(),
                        Symbol::Ap.into(),
                        x.clone(),
                        z.clone(),
                        Symbol::Ap.into(),
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
                    &[
                        Symbol::Ap.into(),
                        Symbol::Ap.into(),
                        x.clone(),
                        z.clone(),
                        y.clone(),
                    ],
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
                    &[
                        Symbol::Ap.into(),
                        x.clone(),
                        Symbol::Ap.into(),
                        y.clone(),
                        z.clone(),
                    ],
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
                Symbol::Pair(x.clone().into(), y.clone().into()).into()
            } else {
                unreachable!()
            }
        }
        Symbol::Car => op1(&operands, |op| match op {
            Symbol::Pair(v1, _) => v1.clone(),
            Symbol::List(_) => unreachable!("List should have been lowered"),
            _ => unreachable!("Mod with invalid operands"),
        }),

        Symbol::Cdr => op1(&operands, |op| match op {
            Symbol::Pair(_, v2) => v2.clone(),
            Symbol::List(_) => unreachable!("List should have been lowered"),
            _ => unreachable!("Mod with invalid operands"),
        }),

        Symbol::Nil => Symbol::Nil.into(),

        Symbol::IsNil => op1(&operands, |op| {
            if op == &Symbol::Nil {
                Symbol::T.into()
            } else {
                Symbol::F.into()
            }
        }),

        Symbol::List(_) => unreachable!("List should have been lowered"),

        // Symbol::Draw => {},
        Symbol::Checkerboard => lit2(operands, |x, y| {
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
                if literal.deref() == &Symbol::Lit(0) {
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

fn eval_thunk(instruction: &SymbolCell, vars: &Environment) -> SymbolCell {
    match instruction.deref() {
        Symbol::PartFn(op, operands, 0) => eval_val(op.clone(), operands.clone(), vars),
        _ => instruction.clone(),
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
    let mut stack = VecDeque::<SymbolCell>::new();

    let lowered_symbols: Vec<SymbolCell> = lower_symbols(instructions);

    for inst in lowered_symbols.iter().rev() {
        match inst.deref() {
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

            _ => stack.push_back(eval_thunk(&inst, vars)),
        }
    }

    assert_eq!(stack.len(), 1);

    eval_thunk(&stack.pop_back().unwrap(), vars)
}

#[cfg(test)]
mod tests;
