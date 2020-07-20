// https://message-from-space.readthedocs.io/en/latest/message7.html

use std::fmt::{Debug, Formatter, Result};
use std::ops::Deref;
use std::rc::Rc;

use image::GrayImage;

pub use modulations::{demodulate_string, modulate_to_string};

pub type Number = i64;

#[derive(Clone, Eq, PartialEq)]
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

impl Debug for SymbolCell {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:?}", self.0)
    }
}

impl Debug for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        fn limited(symbol: &Symbol, f: &mut Formatter<'_>, depth: i8) -> Result {
            if depth <= 0 {
                return write!(f, "..more..");
            }

            match symbol {
                Symbol::T => write!(f, "t"),
                Symbol::Lit(v) => write!(f, "{}", v),
                Symbol::Eq => write!(f, "eq"),
                Symbol::Inc => write!(f, "inc"),
                Symbol::Dec => write!(f, "dec"),
                Symbol::Add => write!(f, "add"),
                Symbol::Var(id) => write!(f, "{:?}", id),
                Symbol::Mul => write!(f, "mul"),
                Symbol::Div => write!(f, "div"),
                Symbol::F => write!(f, "f"),
                Symbol::Lt => write!(f, "lt"),
                Symbol::Mod => write!(f, "mod"),
                Symbol::Dem => write!(f, "dem"),
                Symbol::Send => write!(f, "send"),
                Symbol::Neg => write!(f, "neg"),
                Symbol::Ap => write!(f, "ap"),
                Symbol::S => write!(f, "s"),
                Symbol::C => write!(f, "c"),
                Symbol::B => write!(f, "b"),
                Symbol::Pwr2 => write!(f, "pwr2"),
                Symbol::I => write!(f, "i"),
                Symbol::Cons => write!(f, "cons"),
                Symbol::Car => write!(f, "car"),
                Symbol::Cdr => write!(f, "cdr"),
                Symbol::Nil => write!(f, "nil"),
                Symbol::IsNil => write!(f, "isnil"),
                Symbol::Draw => write!(f, "draw"),
                Symbol::Checkerboard => write!(f, "checkerboard"),
                Symbol::MultipleDraw => write!(f, "multipledraw"),
                Symbol::If0 => write!(f, "if0"),
                Symbol::Interact => write!(f, "interact"),
                Symbol::StatelessDraw => write!(f, "statelessdraw"),

                Symbol::Modulated(m) => {
                    write!(f, "modulated(")?;
                    write!(f, "{:?}", m)?;
                    write!(f, ")")
                }
                Symbol::List(items) => {
                    write!(f, "[")?;
                    for item in items.iter().take(10) {
                        limited(item, f, depth - 1)?;
                        write!(f, ", ")?;
                    }
                    if items.len() > 10 {
                        write!(f, "...")?;
                    }
                    write!(f, "]")
                }
                Symbol::Pair(fst, second) => {
                    write!(f, "pair(")?;

                    limited(fst, f, depth - 1)?;
                    write!(f, ", ")?;
                    limited(second, f, depth - 1)?;
                    write!(f, ")")
                }
                Symbol::Closure { captured_arg, body } => {
                    write!(f, "(")?;
                    limited(body, f, depth - 1)?;
                    write!(f, " ")?;
                    limited(captured_arg, f, depth - 1)?;
                    write!(f, ")")
                }
                Symbol::Image(buffer) => write!(f, "image{:?}", buffer.dimensions()),
                Symbol::StoreArg(arg) => {
                    write!(f, "StoreArg(")?;
                    write!(f, "{:?}, ", arg)?;
                    write!(f, ")")
                }
            }
        }

        limited(self, f, 127)
    }
}

pub mod modulations;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Statement(pub Identifier, pub Vec<Symbol>);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Identifier {
    Name(String),
    PreludeArg(String),
}

impl Identifier {
    pub fn id(id: usize) -> Self {
        Identifier::Name(format!(":{}", id))
    }
}

impl From<usize> for Identifier {
    fn from(value: usize) -> Self {
        Identifier::id(value)
    }
}

#[derive(Clone, Eq, PartialEq)]
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
    Var(Identifier),
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
    Pair(SymbolCell, SymbolCell),
    Closure {
        captured_arg: SymbolCell,
        body: SymbolCell,
    },
    Modulated(modulations::Modulated),
    Image(GrayImage),
    StoreArg(Identifier),
}

impl Symbol {
    pub fn num_args(self: &Symbol) -> i8 {
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
            Symbol::Pair(_, _) => 0,
            Symbol::Modulated(_) => 0,
            Symbol::Closure { .. } => 1,
            Symbol::Image(_) => 0,
            Symbol::StoreArg(_args) => 2,
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

pub fn lower_symbols<T>(symbols: &[T]) -> Vec<SymbolCell>
where
    T: Into<SymbolCell> + Clone,
{
    symbols
        .iter()
        .map(|inst| inst.clone().into().canonicalize().into())
        .collect()
}

#[cfg(test)]
mod tests;
