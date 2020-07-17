use crate::ast::{Identifier, Symbol};
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;
use std::str::FromStr;
use tokio::stream::StreamExt;

#[derive(Parser)]
#[grammar = "parser/grammar.pest"] // relative to src
struct ProgramParser;

fn parse_as_lines(input: &str) -> HashMap<Identifier, Symbol> {
    let lines = input.split('\n');
    let mut map = HashMap::new();
    for line in lines {
        let parsed_line = ProgramParser::parse(Rule::line, line)
            .expect("failed to parse line")
            .next()
            .unwrap();

        let assignment: Pairs<'_, _> = parsed_line.into_inner();
        let assignment = assignment.peek().unwrap().into_inner();

        let id = assignment.peek().unwrap();
        let id = id.into_inner().peek().unwrap();

        let id = match id.as_rule() {
            Rule::var => {
                Identifier::Var(usize::from_str(id.into_inner().peek().unwrap().as_str()).unwrap())
            }
            Rule::identifier => Identifier::Name(id.as_str().to_string()),
            _ => unimplemented!("Invalid variable id {:?}", id),
        };

        dbg!(&id);

        let symbols: Vec<Symbol> = assignment
            .skip(1)
            .map(|pair| match pair.as_rule() {
                Rule::ap => Symbol::Ap,
                Rule::var => {
                    let value = pair.into_inner().as_str();
                    Symbol::Var(usize::from_str(&value).unwrap())
                }
                Rule::cons => Symbol::Cons,
                Rule::number => Symbol::Lit(i64::from_str(pair.as_str()).unwrap()),
                Rule::nil => Symbol::Nil,
                Rule::eq => Symbol::Eq,
                _ => unimplemented!("Unhandled Pair {:?}", pair),
            })
            .collect();

        dbg!(&symbols);

        map.insert(id, Symbol::List(symbols));
    }

    map
}

#[cfg(test)]
mod tests;
