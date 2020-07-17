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
        let ass = assignment.peek().unwrap().into_inner();

        let id = ass.peek().unwrap();
        dbg!(id.as_str());

        let symbols: Vec<Symbol> = ass
            .skip(1)
            .map(|pair| match pair.as_rule() {
                Rule::ap => Symbol::Ap,
                Rule::var => {
                    Symbol::Var(usize::from_str(&pair.into_inner().as_str()[1..]).unwrap())
                }
                Rule::cons => Symbol::Cons,
                Rule::number => Symbol::Lit(i64::from_str(dbg!(pair.as_str().trim())).unwrap()),
                Rule::nil => Symbol::Nil,
                _ => unimplemented!("Unhandled Pair {:?}", pair),
            })
            .collect();
        dbg!(&symbols);

        map.insert(
            Identifier::Name(id.as_str().to_string()),
            Symbol::List(symbols),
        );
    }

    map
}

#[cfg(test)]
mod tests;
