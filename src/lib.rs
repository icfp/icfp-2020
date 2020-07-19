use crate::ast::Symbol;
use std::ops::Deref;

pub mod ast;
pub mod client;
pub mod parser;
pub mod stack_interpreter;

pub fn run<T: Into<String>>(program: T) -> Symbol {
    let statements = parser::parse_as_lines(program.into().as_str());

    ast::interpret(dbg!(statements))
}

#[cfg(test)]
mod tests;
