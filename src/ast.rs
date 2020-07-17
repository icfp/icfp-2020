// https://message-from-space.readthedocs.io/en/latest/message7.html

pub enum Symbol {
    Number(i16), // 1-3
    Eq(Box<Symbol>, Box<Symbol>), // 4
    Inc(Box<Symbol>), // 5
    Dec(Box<Symbol>), // 6
    Add(Box<Symbol>, Box<Symbol>), // 7
    Var(i16), // 8
    Mul(Box<Symbol>, Box<Symbol>), // 9
}

pub fn eval(tree: Symbol) -> i32 {
    match tree {
        Symbol::Number(i) => i as i32,
        Symbol::Eq(x, y) => (eval(*x) == eval(*y)) as i32,
        Symbol::Inc(x) => eval(*x) + 1,
        Symbol::Dec(x) => eval(*x) - 1,
        Symbol::Add(x, y) => eval(*x) + eval(*y),
        Symbol::Var(_) => panic!(),
        Symbol::Mul(x, y) => eval(*x) * eval(*y),
    }
}