use std::collections::HashMap;

use crate::parser::{AST, Operation};

pub fn evaluate(ast: AST, vars: &mut HashMap<String, i32>) -> i32 {
    match ast.op {
        Operation::ADD => evaluate(*ast.left.unwrap(), vars) + evaluate(*ast.right.unwrap(), vars),
        Operation::SUBTRACT => {
            evaluate(*ast.left.unwrap(), vars) - evaluate(*ast.right.unwrap(), vars)
        }
        Operation::MULTIPLY => {
            evaluate(*ast.left.unwrap(), vars) * evaluate(*ast.right.unwrap(), vars)
        }
        Operation::DIVIDE => {
            evaluate(*ast.left.unwrap(), vars) / evaluate(*ast.right.unwrap(), vars)
        }
        Operation::MODULO => {
            evaluate(*ast.left.unwrap(), vars) % evaluate(*ast.right.unwrap(), vars)
        }
        Operation::ASSIGN => {
            let left = *ast.left.unwrap();
            if let Operation::IDENT(v) = left.op {
                let right = evaluate(*ast.right.unwrap(), vars);
                vars.insert(v.clone(), right);
                *vars.get(&v).unwrap_or_else(|| unreachable!())
            } else {
                panic!("left is not IDENT");
            }
        }
        Operation::NUMBER(v) => v,
        Operation::IDENT(v) => {
            if let Some(v) = vars.get(&v) {
                *v
            } else {
                panic!("{0} is not defined", v)
            }
        }
    }
}
