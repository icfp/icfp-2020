use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Mutex;

use crate::ast::lower_symbols;
use crate::ast::modulations;

use super::ast::{Identifier, Number, Statement, Symbol, SymbolCell};
use image::{GrayImage, ImageFormat};
use std::mem;
use std::time::SystemTime;

type StackEnvironment = HashMap<Identifier, SymbolCell>;
type RuntimeStack = Vec<SymbolCell>;

trait Effects {
    fn send(&self, content: String) -> String;
    fn display(&self, image: &GrayImage);
}

struct NullEffects();

impl Effects for NullEffects {
    fn send(&self, content: String) -> String {
        content
    }

    fn display(&self, _image: &GrayImage) {
        // do nothing
    }
}

pub struct VM {
    heap: StackEnvironment,
    stack: RuntimeStack,
    effects: Box<dyn Effects>,
}

impl VM {
    pub fn new() -> Mutex<Self> {
        Mutex::new(VM {
            heap: StackEnvironment::new(),
            stack: RuntimeStack::new(),
            effects: Box::from(NullEffects()),
        })
    }

    fn pop(&mut self) -> SymbolCell {
        self.stack.pop().unwrap()
    }

    fn push(&mut self, symbol: SymbolCell) {
        self.stack.push(symbol)
    }

    fn var(&self, id: Identifier) -> SymbolCell {
        self.heap
            .get(&id)
            .expect(&format!("Can't find {:?} in {:?}", id, self.heap.keys()))
            .clone()
    }
}

pub trait Resolve {
    fn resolve(&self, symbol: &SymbolCell) -> SymbolCell;
    fn pop(&self) -> SymbolCell;
    fn push(&self, symbol: SymbolCell);
    fn var(&self, id: Identifier) -> SymbolCell;
}

impl Resolve for Mutex<VM> {
    fn resolve(&self, symbol: &SymbolCell) -> SymbolCell {
        dbg!(run_expression(symbol.clone(), self))
    }

    fn pop(&self) -> SymbolCell {
        self.lock().unwrap().pop()
    }

    fn push(&self, symbol: SymbolCell) {
        self.lock().unwrap().push(symbol)
    }

    fn var(&self, id: Identifier) -> SymbolCell {
        self.lock().unwrap().var(id)
    }
}

fn build_symbol_tree(statement: &Statement) -> SymbolCell {
    let mut stack = Vec::<SymbolCell>::new();

    let lowered_symbols: Vec<SymbolCell> = lower_symbols(&statement.1);

    for inst in lowered_symbols.iter().rev() {
        let val = lower_applies(inst, &mut stack);
        stack.push(val);
    }

    assert_eq!(
        stack.len(),
        1,
        "Stack contains more than one item: {:?}",
        stack
    );

    dbg!(stack).pop().unwrap()
}

fn lower_applies(op: &SymbolCell, operands: &mut Vec<SymbolCell>) -> SymbolCell {
    match op.deref() {
        Symbol::Ap => {
            dbg!(&operands);
            let fun = operands.pop().unwrap();
            let arg = operands.pop().unwrap();
            Symbol::Closure {
                captured_arg: arg,
                body: fun,
            }
            .into()
        }
        _ => op.clone(),
    }
}

fn op1<F>(vm: &Mutex<VM>, f: F) -> SymbolCell
where
    F: FnOnce(SymbolCell) -> SymbolCell,
{
    let op = vm.pop();

    f(op)
}

fn op2<F>(vm: &Mutex<VM>, f: F) -> SymbolCell
where
    F: FnOnce(SymbolCell, SymbolCell) -> SymbolCell,
{
    let op1 = vm.pop();
    let op2 = vm.pop();

    f(op1, op2)
}

fn op3<F>(vm: &Mutex<VM>, f: F) -> SymbolCell
where
    F: FnOnce(SymbolCell, SymbolCell, SymbolCell) -> SymbolCell,
{
    let op1 = vm.pop();
    let op2 = vm.pop();
    let op3 = vm.pop();

    f(op1, op2, op3)
}

fn stack_lit1<T: Into<SymbolCell>, F: FnOnce(Number) -> T>(vm: &Mutex<VM>, f: F) -> SymbolCell {
    op1(vm, |arg| match vm.resolve(&arg).deref() {
        Symbol::Lit(x) => f(*x).into(),
        arg => unreachable!("Non-literal operand: {:?}", arg),
    })
}

fn stack_lit2<T: Into<SymbolCell>, F: FnOnce(Number, Number) -> T>(
    vm: &Mutex<VM>,
    f: F,
) -> SymbolCell {
    op2(vm, |first, second| {
        match (vm.resolve(&first).deref(), vm.resolve(&second).deref()) {
            (Symbol::Lit(x), Symbol::Lit(y)) => f(*x, *y).into(),
            args => unreachable!("Non-literal operands: {:?}", args),
        }
    })
}

struct SymbolIter<'a> {
    vm: &'a Mutex<VM>,
    symbol: SymbolCell,
}

impl Iterator for SymbolIter<'_> {
    type Item = SymbolCell;

    fn next(&mut self) -> Option<Self::Item> {
        let symbol = mem::replace(&mut self.symbol, Symbol::Nil.into());

        match symbol.deref() {
            Symbol::Nil => None,

            Symbol::Pair(hd, tl) => {
                self.symbol = tl.clone();
                Some(self.vm.resolve(hd))
            }

            _ => {
                self.symbol = Symbol::Nil.into();
                Some(self.vm.resolve(&symbol))
            }
        }
    }
}

fn iter_symbols(vm: &Mutex<VM>, symbol: SymbolCell) -> SymbolIter {
    SymbolIter { vm, symbol }
}

pub fn run_function(function: SymbolCell, vm: &Mutex<VM>) {
    let result = match function.deref() {
        Symbol::Var(id) => dbg!(vm.var(id.clone()).clone()),
        Symbol::Lit(_) => function.clone(),
        Symbol::Pair(_, _) => function.clone(),
        Symbol::Modulated(_) => function.clone(),
        Symbol::Image(_) => function.clone(),
        Symbol::Inc => stack_lit1(vm, |x| x + 1),
        Symbol::Dec => stack_lit1(vm, |x| x - 1),
        Symbol::Add => stack_lit2(vm, |x, y| x + y),
        Symbol::Mul => stack_lit2(vm, |x, y| x * y),
        Symbol::Div => stack_lit2(vm, |x, y| x / y),
        Symbol::If0 => op3(vm, |test, first, second| {
            if vm.resolve(&test).deref() == &Symbol::Lit(0) {
                first
            } else {
                second
            }
        }),

        Symbol::Eq => op2(vm, |x, y| {
            let x = vm.resolve(&x);
            let y = vm.resolve(&y);
            if x.deref() == y.deref() {
                Symbol::T.into()
            } else {
                Symbol::F.into()
            }
        }),

        Symbol::Lt => stack_lit2(vm, |x, y| if x < y { Symbol::T } else { Symbol::F }),

        Symbol::T => op2(vm, |x, _| x),

        Symbol::F => op2(vm, |_, y| y),

        Symbol::Mod => op1(vm, |op| {
            let vec = modulations::modulate(&op, vm);
            Symbol::Modulated(vec).into()
        }),

        Symbol::Dem => op1(vm, |op| match vm.resolve(&op).deref() {
            Symbol::Modulated(val) => modulations::demodulate(val.clone()).into(),
            _ => unreachable!("Dem with invalid operands"),
        }),
        // Symbol::Send => {},
        Symbol::Neg => stack_lit1(vm, |x| Symbol::Lit(-x.clone())),

        Symbol::Pwr2 => stack_lit1(vm, |x| i64::pow(2, x as u32)),
        Symbol::I => op1(vm, |op| op.clone()),

        Symbol::Cons => op2(vm, |op1, op2| Symbol::Pair(op1.clone(), op2.clone()).into()),
        Symbol::Car => op1(vm, |op| match vm.resolve(&op).deref() {
            Symbol::Pair(hd, _) => vm.resolve(hd),
            Symbol::List(_) => unreachable!("List should have been lowered"),
            op => unreachable!("Car with invalid operands: {:?}", op),
        }),
        Symbol::Cdr => op1(vm, |op| {
            match vm.resolve(&op).deref() {
                // this one should be lazy and not resolve, i think...
                Symbol::Pair(_, tail) => tail.clone(),
                Symbol::List(_) => unreachable!("List should have been lowered"),
                op => unreachable!("Cdr with invalid operands: {:?}", op),
            }
        }),
        Symbol::Nil => Symbol::Nil.into(),
        Symbol::IsNil => op1(vm, |op| {
            if vm.resolve(&op).deref() == &Symbol::Nil {
                Symbol::T.into()
            } else {
                Symbol::F.into()
            }
        }),

        Symbol::Draw => op1(vm, |x| {
            let mut image = GrayImage::new(640, 480);
            for sym in iter_symbols(vm, x) {
                match sym.deref() {
                    Symbol::Pair(x, y) => {
                        let x = vm.resolve(x);
                        let y = vm.resolve(y);
                        match (x.deref(), y.deref()) {
                            (&Symbol::Lit(x), &Symbol::Lit(y)) => {
                                image.put_pixel(x as u32, y as u32, [255u8].into())
                            }
                            _ => panic!(),
                        }
                    }
                    _ => panic!(),
                }
            }

            let name = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap();

            image
                .save_with_format(format!("/tmp/{}.png", name.as_secs()), ImageFormat::Png)
                .unwrap();

            Symbol::Image(image).into()
        }),

        Symbol::Checkerboard => stack_lit2(vm, |x, y| {
            let mut image = GrayImage::new(x as u32, y as u32);
            for x in 0..x as u32 {
                for y in 0..y as u32 {
                    let color = ((x % 2) ^ (y % 2)) as u8;
                    image.put_pixel(x, y, [255u8 * color].into())
                }
            }

            vm.lock().unwrap().effects.deref().display(&image);

            let name = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap();

            image
                .save_with_format(format!("/tmp/{}.png", name.as_secs()), ImageFormat::Png)
                .unwrap();

            // let name = SystemTime::now()
            //     .duration_since(SystemTime::UNIX_EPOCH)
            //     .unwrap();
            //
            // image
            //     .save_with_format(format!("/tmp/{}.png", name.as_secs()), ImageFormat::Png)
            //     .unwrap();
            //
            Symbol::Image(image)
        }),
        Symbol::S => {
            // https://en.wikipedia.org/wiki/SKI_combinator_calculus
            // Sxyz = xz(yz)
            op3(vm, |x, y, z| {
                Symbol::Closure {
                    captured_arg: Symbol::Closure {
                        captured_arg: z.clone(),
                        body: y,
                    }
                    .into(),
                    body: Symbol::Closure {
                        captured_arg: z,
                        body: x,
                    }
                    .into(),
                }
                .into()
            })
        }
        Symbol::Closure {
            captured_arg: arg,
            body: fun,
        } => {
            vm.push(arg.clone());
            fun.clone()
        }
        Symbol::C => {
            // https://en.wikipedia.org/wiki/B,_C,_K,_W_system
            // C x y z = x z y

            op3(vm, |x, y, z| {
                let xz_apply = Symbol::Closure {
                    captured_arg: z,
                    body: x,
                };

                let c = Symbol::Closure {
                    captured_arg: y,
                    body: xz_apply.into(),
                }
                .into();
                c
            })
        }

        Symbol::B => {
            // https://en.wikipedia.org/wiki/B,_C,_K,_W_system
            // B x y z = x (y z)

            op3(vm, |x, y, z| {
                Symbol::Closure {
                    captured_arg: Symbol::Closure {
                        captured_arg: z,
                        body: y,
                    }
                    .into(),
                    body: x,
                }
                .into()
            })
        }
        Symbol::StoreArg(name) => {
            let func = vm.pop();
            let value_to_store = vm.pop();
            vm.lock().unwrap().heap.insert(name.clone(), value_to_store);
            func // intentional
        }
        func => unimplemented!("Function not supported: {:?}", func),
    };

    vm.push(dbg!(result));
}

pub fn run_expression(symbol: SymbolCell, vm: &Mutex<VM>) -> SymbolCell {
    let mut op: SymbolCell = symbol;
    let mut count = 0;
    loop {
        dbg!(&op);
        // dbg!(&vm.stack);
        if count > 1000 {
            panic!();
        }
        count += 1;
        run_function(op.clone(), vm);
        let sym: SymbolCell = dbg!(vm.pop());
        match sym.deref() {
            // :3 = cons
            Symbol::Closure { .. } => op = sym.clone(),
            _ if op != sym && vm.lock().unwrap().stack.len() >= sym.num_args() as usize => {
                op = sym.clone()
            }
            _ => {
                // :-(
                return vm
                    .lock()
                    .unwrap()
                    .stack
                    .iter()
                    .rev()
                    .take(sym.num_args() as usize)
                    .fold(sym, |acc, el| {
                        Symbol::Closure {
                            captured_arg: el.clone(),
                            body: acc,
                        }
                        .into()
                    });
            }
        }
    }
}

pub fn run(symbol: SymbolCell, environment: &StackEnvironment) -> SymbolCell {
    let vm = VM {
        stack: RuntimeStack::new(),
        heap: environment.clone(),
        effects: Box::from(NullEffects()),
    };
    run_expression(symbol, &Mutex::new(vm))
}

pub fn stack_interpret(statements: Vec<Statement>) -> Symbol {
    let mut env = StackEnvironment::new();
    let last_statement_id = statements.last().unwrap().0.clone();

    for statement in statements.clone() {
        let statements_rvalue = dbg!(build_symbol_tree(&statement));

        env.insert(statement.0, statements_rvalue);
    }

    let last_rvalue = env[&last_statement_id].clone();

    let result = run(last_rvalue, &env);

    result.deref().clone()
}

pub fn eval_instructions<T: Into<Symbol> + Clone>(symbols: &[T]) -> Symbol {
    let instructions: Vec<Symbol> = symbols.iter().map(|sym| sym.clone().into()).collect();
    let statement = Statement(Identifier::Name("foo".to_string()), instructions);

    stack_interpret(vec![statement])
}

#[cfg(test)]
mod stack_tests {
    use crate::ast::{Statement, Symbol};
    use crate::parser::parse_as_lines;
    use crate::stack_interpreter::build_symbol_tree;

    use super::stack_interpret;
    use super::Symbol::*;

    fn run_lines(lines: Vec<Statement>, expectation: Symbol) {
        let symbol = dbg!(stack_interpret(lines));
        assert_eq!(symbol, expectation);
    }

    fn run_test(program: &str, expectation: Symbol) {
        let lines = parse_as_lines(program);
        run_lines(lines, expectation)
    }

    #[test]
    fn inc() {
        let lines = parse_as_lines(":1 = ap inc 1");
        let _symbol = dbg!(stack_interpret(lines));
    }

    #[test]
    fn add() {
        let lines = parse_as_lines(":1 = ap ap add 2 1");
        let symbol = dbg!(stack_interpret(lines));
        assert_eq!(symbol, Lit(3))
    }

    #[test]
    fn mul() {
        // https://message-from-space.readthedocs.io/en/latest/message9.html

        /*
        ap ap mul 4 2   =   8
        ap ap mul 3 4   =   12
        ap ap mul 3 -2   =   -6
        ap ap mul x0 x1   =   ap ap mul x1 x0
        ap ap mul x0 0   =   0
        ap ap mul x0 1   =   x0
        */
        run_test(":1 = ap ap mul 4 2", Lit(8));
        run_test(":1 = ap ap mul 3 4", Lit(12));
        run_test(":1 = ap ap mul 3 -2", Lit(-6));
        run_test(":1 = ap ap mul -2 3", Lit(-6));
        run_test(":1 = ap ap mul 4 0", Lit(0));
        run_test(":1 = ap ap mul 4 1", Lit(4));
    }

    #[test]
    fn less_than() {
        // message 12
        /*
        ap ap lt 0 -1   =   f
        ap ap lt 0 0   =   f
        ap ap lt 0 1   =   t
        ap ap lt 0 2   =   t
        ...
        ap ap lt 1 0   =   f
        ap ap lt 1 1   =   f
        ap ap lt 1 2   =   t
        ap ap lt 1 3   =   t
        ...
        ap ap lt 2 1   =   f
        ap ap lt 2 2   =   f
        ap ap lt 2 3   =   t
        ap ap lt 2 4   =   t
        ...
        ap ap lt 19 20   =   t
        ap ap lt 20 20   =   f
        ap ap lt 21 20   =   f
        ...
        ap ap lt -19 -20   =   f
        ap ap lt -20 -20   =   f
        ap ap lt -21 -20   =   t
        */
        run_test(":1 = ap ap lt 0 -1", F);
        run_test(":1 = ap ap lt 0 0", F);
        run_test(":1 = ap ap lt 0 1", T);
    }

    #[test]
    fn div() {
        // https://message-from-space.readthedocs.io/en/latest/message10.html

        /*
        ap ap div 4 2   =   2
        ap ap div 4 3   =   1
        ap ap div 4 4   =   1
        ap ap div 4 5   =   0
        ap ap div 5 2   =   2
        ap ap div 6 -2   =   -3
        ap ap div 5 -3   =   -1
        ap ap div -5 3   =   -1
        ap ap div -5 -3   =   1
        ap ap div x0 1   =   x0
        */

        run_test(":1 = ap ap div 4 2", Lit(2));
        run_test(":1 = ap ap div 4 3", Lit(1));
        run_test(":1 = ap ap div 4 5", Lit(0));
        run_test(":1 = ap ap div 5 2", Lit(2));
        run_test(":1 = ap ap div 6 -2", Lit(-3));
        run_test(":1 = ap ap div 5 -3", Lit(-1));
        run_test(":1 = ap ap div -5 3", Lit(-1));
        run_test(":1 = ap ap div -5 -3", Lit(1));
    }

    #[test]
    fn equality() {
        run_test(":1 = ap ap eq 1 1", T);
    }

    #[test]
    fn inequality() {
        run_test(":1 = ap ap eq 1 2", F);
    }

    #[test]
    fn cons() {
        run_test(":1 = ap ap cons 1 2", Pair(Lit(1).into(), Lit(2).into()));
    }

    #[test]
    fn car() {
        run_test(":1 = ap car ap ap cons 1 2", Lit(1));
    }

    #[test]
    fn cdr() {
        run_test(":1 = ap cdr ap ap cons 1 2", Lit(2));
    }

    #[test]
    fn true_func() {
        // message 21

        /*
        ap ap t x0 x1   =   x0
        ap ap t 1 5   =   1
        ap ap t t i   =   t
        ap ap t t ap inc 5   =   t
        ap ap t ap inc 5 t   =   6
        */

        run_test(":1 = ap ap t 1 2", Lit(1));
        run_test(":1 = ap ap t t i", T);
        run_test(":1 = ap ap t t ap inc 5", T);
        run_test(":1 = ap ap t ap inc 5 t", Lit(6));
    }

    #[test]
    fn var_literal_lookahead() {
        let lines = parse_as_lines(
            ":1 = ap ap add 2 :2
                                                    :2 = 3
                                                    :3 = :1",
        );
        let _symbol = dbg!(stack_interpret(lines));
    }

    #[test]
    fn var_curry() {
        let lines = parse_as_lines(
            ":1 = ap add 2
             :2 = 3
             :3 = ap :1 :2",
        );
        let _symbol = dbg!(stack_interpret(lines));
    }

    #[test]
    fn var_dead_code() {
        let lines = parse_as_lines(
            ":0 = 5
            :1 = ap ap ap if0 1 :1 :0",
        );
        let _symbol = dbg!(stack_interpret(lines));
    }

    #[test]
    fn s_combinator() {
        let lines = parse_as_lines(":1 = ap ap ap s add inc 1");
        let symbol = dbg!(stack_interpret(lines));
        assert_eq!(symbol, Lit(3));

        let lines = parse_as_lines(":2 = ap ap ap s mul ap add 1 6");
        let symbol = dbg!(stack_interpret(lines));
        assert_eq!(symbol, Lit(42));
    }

    #[test]
    fn c_combinator() {
        let lines = parse_as_lines(":1 = ap ap ap c add 1 2");
        let symbol = dbg!(stack_interpret(lines));
        assert_eq!(symbol, Lit(3));
    }

    #[test]
    fn b_combinator() {
        let lines = parse_as_lines(
            ":0 = 42
        :1 = ap ap ap b inc dec :0",
        );
        let symbol = dbg!(stack_interpret(lines));
        assert_eq!(symbol, Lit(42));
    }

    #[test]
    fn test_lookahead() {
        let lines = parse_as_lines(
            ":1 = ap add 1
             :2 = ap :1 2",
        );

        let symbol = dbg!(stack_interpret(lines));
        assert_eq!(symbol, Lit(3));
    }

    #[test]
    fn run_simple() {
        let lines = parse_as_lines(
            ":1 = ap add 1
:2 = ap ap ap ap if0 1 :2 :1 2
:3 = :2",
        );

        let symbol = dbg!(stack_interpret(lines));
        assert_eq!(symbol, Lit(3));
    }

    #[test]
    #[ignore]
    fn galaxy_first_pass() {
        run_test(
            ":1338 = ap ap c ap ap b c ap ap c ap ap b c 1 2 3
             galaxy = :1338",
            Symbol::C, // not sure what the right answer should be...
                       // but seems it could be 1 or C
                       // was C when galaxy was running...
        )
    }

    #[test]
    fn interact() {
        // ap modem x0 = ap dem ap mod x0
        // ap ap f38 x2 x0 = ap ap ap if0 ap car x0 ( ap modem ap car ap cdr x0 , ap multipledraw ap car ap cdr ap cdr x0 ) ap ap ap interact x2 ap modem ap car ap cdr x0 ap send ap car ap cdr ap cdr x0
        // ap ap ap interact x2 x4 x3 = ap ap f38 x2 ap ap x2 x4 x3

        run_test(
            "modem = ap dem mod
               :1 = ap modem 1",
            Lit(1),
        );
    }

    #[test]
    #[ignore]
    fn prelude() {
        // ap modem x0 = ap dem ap mod x0
        // ap ap f38 x2 x0 = ap ap ap if0 ap car x0 ( ap modem ap car ap cdr x0 , ap multipledraw ap car ap cdr ap cdr x0 ) ap ap ap interact x2 ap modem ap car ap cdr x0 ap send ap car ap cdr ap cdr x0
        // ap ap ap interact x2 x4 x3 = ap ap f38 x2 ap ap x2 x4 x3
        let mut lines = crate::parser::parse_as_lines(include_str!("ast/prelude-small.txt"));
        // ap ap ap interact x0 nil ap ap vec 0 0 = ( x16 , ap multipledraw x64 )
        // ap ap ap interact x0 x16 ap ap vec x1 x2 = ( x17 , ap multipledraw x65 )
        // ap ap ap interact x0 x17 ap ap vec x3 x4 = ( x18 , ap multipledraw x66 )
        // ap ap ap interact x0 x18 ap ap vec x5 x6 = ( x19 , ap multipledraw x67 )
        //lines.extend_from_slice(&parse_as_lines("run = ap ap interact galaxy nil ( 0, 0 )"));
        lines.extend_from_slice(&parse_as_lines(":1 = ap modem 1"));
        run_lines(lines, Lit(1));
    }

    #[test]
    #[ignore]
    fn prelude_div() {
        // ap modem x0 = ap dem ap mod x0
        // ap ap f38 x2 x0 = ap ap ap if0 ap car x0 ( ap modem ap car ap cdr x0 , ap multipledraw ap car ap cdr ap cdr x0 ) ap ap ap interact x2 ap modem ap car ap cdr x0 ap send ap car ap cdr ap cdr x0
        // ap ap ap interact x2 x4 x3 = ap ap f38 x2 ap ap x2 x4 x3
        let mut lines = crate::parser::parse_as_lines(include_str!("ast/prelude-small.txt"));
        // ap ap ap interact x0 nil ap ap vec 0 0 = ( x16 , ap multipledraw x64 )
        // ap ap ap interact x0 x16 ap ap vec x1 x2 = ( x17 , ap multipledraw x65 )
        // ap ap ap interact x0 x17 ap ap vec x3 x4 = ( x18 , ap multipledraw x66 )
        // ap ap ap interact x0 x18 ap ap vec x5 x6 = ( x19 , ap multipledraw x67 )
        //lines.extend_from_slice(&parse_as_lines("run = ap ap interact galaxy nil ( 0, 0 )"));
        lines.extend_from_slice(&parse_as_lines(":1 = ap ap customdiv 4 2"));
        run_lines(lines, Lit(2));
    }

    #[test]
    fn build_f83() {
        let value = "f83 = ap ap ap if0 ap car @X0 ( ap modem ap car ap cdr @X0 , ap multipledraw ap car ap cdr ap cdr @X0 ) ap ap ap interact @X2 ap modem ap car ap cdr @X0 ap send ap car ap cdr ap cdr @X0";
        let lines = parse_as_lines(value);
        dbg!(build_symbol_tree(lines.first().unwrap()));
    }
}
