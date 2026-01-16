use std::{collections::HashMap, io};

use madx_language::{evaluator::evaluate, lexer::Lexer, parser::Parser};

mod lexer;

fn main() {
    let mut inp = String::new();
    io::stdin().read_line(&mut inp).expect("fail to read line");
    let lexer = Lexer::new(  inp.as_bytes()).unwrap();
    let mut parser = Parser::new(lexer);
    let exprs = match  parser.parse() {
        Ok(v) => v,
        Err(e) => {
            println!("{0}", e);
            return;
        }
    };
    let mut variables = HashMap::new();
    for expr in exprs {
        println!("{0}", evaluate(expr, &mut variables));
    }
}
