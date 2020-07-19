// https://message-from-space.readthedocs.io/en/latest/message7.html

use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

pub use modulations::{demodulate_string, modulate_to_string};

pub type Number = i64;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SymbolCell(Rc<Symbol>);

impl From<Symbol> for SymbolCell {
    fn from(symbol: Symbol) -> Self {
        SymbolCell(symbol.into())
    }
}

impl From<&Symbol> for SymbolCell {
    fn from(symbol: &Symbol) -> Self {
        SymbolCell(symbol.clone().into())
    }
}

impl Deref for SymbolCell {
    type Target = Symbol;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

type Environment = HashMap<Identifier, Vec<SymbolCell>>;

pub mod modulations;

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
    ApplyPair(SymbolCell, SymbolCell),
    Modulated(modulations::Modulated),
}

impl Symbol {
    fn num_args(self: &Symbol) -> i8 {
        match self {
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
            Symbol::ApplyPair(_, _) => unreachable!(),
        }
    }
}

pub trait Canonicalize {
    fn canonicalize(&self) -> Self;
}

impl Canonicalize for Symbol {
    fn canonicalize(&self) -> Self {
        match self {
            Symbol::List(v) => v.iter().rfold(Symbol::Nil, |acc, v| {
                Symbol::Pair(v.clone().into(), acc.into())
            }),
            _ => self.clone(),
        }
    }
}

impl Canonicalize for SymbolCell {
    fn canonicalize(&self) -> Self {
        let underlying = self.0.deref();
        match underlying {
            Symbol::List(_) => underlying.canonicalize().into(),
            _ => self.clone(),
        }
    }
}

trait Force {
    fn force(&self, env: &Environment) -> Self;
}

impl Force for SymbolCell {
    fn force(&self, env: &Environment) -> Self {
        let underlying = self.0.deref();
        match underlying {
            Symbol::Pair(hd, tl) => Symbol::Pair(hd.force(env), tl.force(env)).into(),
            Symbol::List(_) => unreachable!(),
            _ => force_resolve(&self.clone(), env).clone(),
        }
    }
}

impl Force for Symbol {
    fn force(&self, env: &Environment) -> Self {
        match self {
            Symbol::Pair(hd, tl) => Symbol::Pair(hd.force(env), tl.force(env)),
            Symbol::List(_) => unreachable!(),
            _ => force_resolve(&self.into(), env).deref().clone(),
        }
    }
}

pub fn eval_instructions<T: Into<SymbolCell> + Clone>(symbols: &[T]) -> Symbol {
    let vars = Environment::new();

    let instructions: Vec<SymbolCell> = symbols.iter().map(|sym| sym.clone().into()).collect();

    eval(&instructions, &vars).force(&vars).0.deref().clone()
}

fn op1<F>(operands: &[SymbolCell], f: F) -> SymbolCell
where
    F: FnOnce(&Symbol) -> SymbolCell,
{
    let len = operands.len() - 1;
    let op = operands[len].deref();
    f(op)
}

fn op2<F>(operands: &[SymbolCell], f: F) -> SymbolCell
where
    F: FnOnce(&Symbol, &Symbol) -> SymbolCell,
{
    let len = operands.len() - 1;
    let op1 = operands[len - 1].deref();
    let op2 = operands[len - 0].deref();
    f(op1, op2)
}

impl Into<Symbol> for Number {
    fn into(self) -> Symbol {
        Symbol::Lit(self)
    }
}

impl Into<SymbolCell> for Number {
    fn into(self) -> SymbolCell {
        Symbol::Lit(self).into()
    }
}

fn lit1<T: Into<SymbolCell>>(
    operands: Vec<SymbolCell>,
    env: &Environment,
    f: fn(Number) -> T,
) -> SymbolCell {
    op1(&operands, move |symbol| match symbol.force(env) {
        Symbol::Lit(x) => f(x).into(),
        _ => unreachable!("Non-literal operand: {:?}", symbol),
    })
}

fn lit2<T: Into<SymbolCell>>(
    operands: Vec<SymbolCell>,
    env: &Environment,
    f: fn(Number, Number) -> T,
) -> SymbolCell {
    op2(&operands, |op1, op2| {
        match (op1.force(env), op2.force(env)) {
            (Symbol::Lit(x), Symbol::Lit(y)) => f(x, y).into(),
            _ => unreachable!("Non-literal operands: {:?}", (op1, op2)),
        }
    })
}

fn force_resolve(op: &SymbolCell, vars: &Environment) -> SymbolCell {
    let mut op = op.clone();
    let mut loops = 10000;

    loop {
        if 0 >= loops {
            panic!("Ahh");
        }
        loops -= 1;

        match op.deref() {
            Symbol::PartFn(_, args, 0) => match args.split_first() {
                Some((hd, tl)) => {
                    // dbg!(&args);
                    let res = apply(hd.clone(), tl.to_vec(), vars);
                    // dbg!(&res);
                    op = res;
                }
                _ => unreachable!(),
            },

            Symbol::Var(idx) => {
                op = eval(&vars[&Identifier::Var(*idx)].clone(), vars);
            }

            Symbol::Pair(head, tail) => {
                return Symbol::Pair(head.force(vars).into(), tail.force(vars).into()).into();
            }

            _ => return op.clone(),
        }
    }
}

fn eval_thunks(op: &SymbolCell, operands: &mut Vec<SymbolCell>) -> SymbolCell {
    match op.deref() {
        Symbol::Ap => {
            let fun = operands.pop().unwrap();
            let arg = operands.pop().unwrap();
            let remaining = fun.num_args() - 1;
            Symbol::PartFn(op.clone(), vec![fun, arg], remaining).into()
        }

        // breaks laziness
        // Symbol::Var(idx) => {
        //     let i = dbg!(*idx);
        //     eval(&vars[&Identifier::Var(i)].clone(), vars)
        // }
        _ => op.clone(),
    }
}

fn apply(op: SymbolCell, operands: Vec<SymbolCell>, vars: &Environment) -> SymbolCell {
    // dbg!(&op);
    match op.deref() {
        Symbol::Lit(_) => op,
        Symbol::Eq => {
            if let [lhs, rhs] = operands.as_slice() {
                if lhs.force(vars) == rhs.force(vars) {
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

        Symbol::Var(idx) => eval(&vars[&Identifier::Var(*idx)].clone(), vars),

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
            Symbol::Modulated(modulations::modulate(&op.force(vars))).into()
        }),
        Symbol::Dem => op1(&operands, |op| match op.force(vars) {
            Symbol::Modulated(val) => modulations::demodulate(val.clone()).into(),
            _ => unreachable!("Dem with invalid operands"),
        }),
        // Symbol::Send => {},
        Symbol::Neg => lit1(operands, vars, |x| Symbol::Lit(-x.clone())),

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
        Symbol::Car => op1(&operands, |op| match op.force(vars) {
            Symbol::Pair(v1, _) => v1.clone(),
            Symbol::List(_) => unreachable!("List should have been lowered"),
            op => unreachable!("Car with invalid operands: {:?}", op),
        }),

        Symbol::Cdr => op1(&operands, |op| match op.force(vars) {
            Symbol::Pair(_, v2) => v2.clone(),
            Symbol::List(_) => unreachable!("List should have been lowered"),
            op => unreachable!("Cdr with invalid operands: {:?}", op),
        }),

        Symbol::Nil => Symbol::Nil.into(),

        Symbol::IsNil => op1(&operands, |op| {
            if op.force(vars) == Symbol::Nil {
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
            SymbolCell(
                Symbol::List(
                    x_axis
                        .flat_map(|x| {
                            y_axis.clone().map(move |y| {
                                Symbol::Pair(Symbol::Lit(x).into(), Symbol::Lit(y).into())
                            })
                        })
                        .collect::<Vec<_>>(),
                )
                .into(),
            )
            .canonicalize()
        }),
        // Symbol::MultipleDraw => {},
        Symbol::If0 => {
            if let [literal, x, y] = operands.as_slice() {
                if literal.force(vars).deref() == &Symbol::Lit(0) {
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
        Symbol::PartFn(_, args, remaining) => {
            match args.split_first() {
                Some((hd, tl)) => {
                    // dbg!(&args);
                    let mut args = Vec::new();
                    args.extend_from_slice(tl);
                    let args_start = operands.len() - (*remaining as usize);
                    // dbg!(&operands);
                    // dbg!(args_start);
                    args.extend_from_slice(&operands[args_start..]);
                    let res = apply(hd.clone(), args, vars);
                    // // dbg!(&res);
                    res
                }
                _ => unreachable!(),
            }
            // :1234 = PartFn(Ap, vec![], 0)
            // PartFn(Ap, vec![:1234, x], 0)
        }

        Symbol::S => {
            // https://en.wikipedia.org/wiki/SKI_combinator_calculus
            // Sxyz = xz(yz)

            match operands.as_slice() {
                [x, y, z] => {
                    let fn0 = Symbol::PartFn(
                        Symbol::Ap.into(),
                        vec![x.clone(), z.clone()],
                        x.num_args() - 1,
                    );

                    let fn1 = Symbol::PartFn(
                        Symbol::Ap.into(),
                        vec![y.clone(), z.clone()],
                        y.num_args() - 1,
                    );

                    let remaining = fn0.num_args() - 1 + fn1.num_args();

                    let s =
                        Symbol::PartFn(Symbol::Ap.into(), vec![fn0.into(), fn1.into()], remaining)
                            .into();
                    // dbg!(&s);

                    s
                }

                _ => unreachable!(),
            }
        }

        Symbol::C => {
            // https://en.wikipedia.org/wiki/B,_C,_K,_W_system
            // C x y z = x z y

            if let [x, y, z] = operands.as_slice() {
                let xz_apply = Symbol::PartFn(
                    Symbol::Ap.into(),
                    vec![x.clone(), z.clone()],
                    x.num_args() - 1,
                );

                Symbol::PartFn(
                    Symbol::Ap.into(),
                    vec![xz_apply.into(), y.clone()],
                    x.num_args() - 2,
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

        _ => unimplemented!("{0:?} is not implemented", op),
    }
}

pub fn lower_symbols<T>(symbols: &[T]) -> Vec<SymbolCell>
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
        let val = eval_thunks(inst, &mut stack);
        stack.push(val);
    }
    dbg!(&stack);
    assert_eq!(stack.len(), 1);
    stack.pop().unwrap().force(vars).into()
}

pub fn interpret(statements: Vec<Statement>) -> Symbol {
    let mut env = HashMap::new();

    for statement in statements.clone() {
        env.insert(
            statement.0,
            statement
                .1
                .clone()
                .iter()
                .map(|s| s.clone().into())
                .collect(),
        );
    }

    let symbol = eval(&statements.last().unwrap().1, &env).force(&env);
    symbol.deref().clone()
}

#[cfg(test)]
mod tests;
