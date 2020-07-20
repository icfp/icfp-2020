use std::fmt::Debug;
use std::str::FromStr;

use pest::iterators::Pair;
use pest::iterators::Pairs;
use pest::Parser;
use pest_derive::Parser;

use crate::ast::{Identifier, Statement, Symbol};

#[derive(Parser)]
#[grammar = "parser/grammar.pest"] // relative to src
struct ProgramParser;

fn parse_pair(pair: Pair<'_, Rule>) -> Symbol {
    match pair.as_rule() {
        Rule::ap => Symbol::Ap,
        Rule::var => {
            let var = pair.into_inner().next().unwrap();
            Symbol::Var(match var.as_rule() {
                Rule::prelude_var => Identifier::PreludeArg(var.as_str().to_string()),
                Rule::var_ref | Rule::identifier => Identifier::Name(var.as_str().to_string()),
                _ => unreachable!(),
            })
        }
        Rule::cons => Symbol::Cons,
        Rule::car => Symbol::Car,
        Rule::cdr => Symbol::Cdr,
        Rule::number => Symbol::Lit(i64::from_str(pair.as_str()).unwrap()),
        Rule::nil => Symbol::Nil,
        Rule::eq => Symbol::Eq,
        Rule::lt => Symbol::Lt,
        Rule::neg => Symbol::Neg,
        Rule::inc => Symbol::Inc,
        Rule::dec => Symbol::Dec,
        Rule::s => Symbol::S,
        Rule::c => Symbol::C,
        Rule::b => Symbol::B,
        Rule::i => Symbol::I,
        Rule::t => Symbol::T,
        Rule::f => Symbol::F,
        Rule::mul => Symbol::Mul,
        Rule::add => Symbol::Add,
        Rule::div => Symbol::Div,
        Rule::isnil => Symbol::IsNil,
        Rule::modulate => Symbol::Mod,
        Rule::demodulate => Symbol::Dem,
        Rule::if0 => Symbol::If0,
        Rule::list => {
            let inner: Vec<_> = pair.into_inner().map(|pair| parse_pair(pair)).collect();
            if inner.is_empty() {
                Symbol::Nil
            } else {
                Symbol::List(inner)
            }
        }
        Rule::symbol => parse_pair(pair.into_inner().peek().unwrap()),
        _ => unimplemented!("Unhandled - {:?}", pair),
    }
}

pub fn parse_as_lines(input: &str) -> Vec<Statement> {
    let lines = input.split('\n');
    let mut statements = Vec::new();

    for line in lines {
        let parsed_line = ProgramParser::parse(Rule::line, line)
            .expect(&format!("Failed to parse line: {}", line))
            .next()
            .unwrap();

        let assignment: Pairs<'_, _> = parsed_line.into_inner();
        let assignment = assignment.peek().unwrap().into_inner();

        let id = assignment.peek();
        if !id.is_some() {
            // empty line
            continue;
        }

        let lval = id.unwrap();

        enum LValue {
            VarFunc(Identifier),
            Prelude(Identifier, Vec<Identifier>),
        }

        let id = match lval.as_rule() {
            Rule::var => LValue::VarFunc(Identifier::Name(lval.as_str().to_string())),
            Rule::func_name => {
                let mut pairs = lval.into_inner().into_iter();
                let name = pairs.next().unwrap();
                let name = Identifier::Name(name.as_str().to_string());

                let args = pairs
                    .next()
                    .expect("expected arg names")
                    .into_inner()
                    .map(|p| Identifier::PreludeArg(p.as_str().to_string()))
                    .collect();

                LValue::Prelude(name, args)
            }

            _ => unimplemented!("Invalid variable id {:?}", lval),
        };

        let symbols: Vec<Symbol> = assignment
            .skip(1) // Skips the lvalue
            .map(|pair| parse_pair(pair))
            .collect();

        impl LValue {
            fn make_statement(self, symbols: Vec<Symbol>) -> Statement {
                match self {
                    LValue::Prelude(name, args) => {
                        let mut body = vec![Symbol::LoadPreludeArgs(args)];
                        body.extend_from_slice(&symbols);
                        Statement(name, body)
                    }
                    LValue::VarFunc(name) => Statement(name, symbols),
                }
            }
        }

        statements.push(id.make_statement(symbols))
    }

    statements
}

#[cfg(test)]
mod tests;
