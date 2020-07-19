use crate::ast::Symbol;
use std::ops::Deref;

pub mod ast;
pub mod client;
pub mod parser;

pub fn run<T: Into<String>>(program: T) -> Symbol {
    let statements = parser::parse_as_lines(program.into().as_str());

    ast::interpret(statements)
}

#[cfg(test)]
mod tests;
