use std::{collections::HashMap, io::{self, Write}};

use madx_language::{evaluator::evaluate, lexer::Lexer, parser::Parser};

mod lexer;

fn main() {
    let mut variables = HashMap::new();

    loop {
        let mut inp = String::new();
        
        print!(">>> ");
        io::stdout().flush().expect("failed to flush");

        io::stdin().read_line(&mut inp).expect("failed to read line");

        if inp.trim() == "exit" {
            break;
        }

        let lexer = Lexer::new(inp.as_bytes()).unwrap();
        let mut parser = Parser::new(lexer);
        let exprs = match parser.parse() {
            Ok(v) => v,
            Err(e) => {
                println!("{0}", e);
                continue;
            }
        };
        
        for expr in exprs {
            println!("{0}", evaluate(expr, &mut variables));
        }
    }
}
