use crate::ast::Symbol;

pub mod ast;
pub mod client;
pub mod decode;
pub mod parser;

pub fn run<T: Into<String>>(program: T) -> Symbol {
    let statements = parser::parse_as_lines(program.into().as_str());

    ast::interpret(statements)
}

#[cfg(test)]
mod tests;
