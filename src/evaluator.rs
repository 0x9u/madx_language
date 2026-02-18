use std::collections::HashMap;

use ordered_float::OrderedFloat;

use crate::parser::{AST, Operation};

type X = OrderedFloat<f32>;

pub fn evaluate(ast: AST, vars: &mut HashMap<String, X>) -> X {
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
        Operation::LSHIFT => {
            OrderedFloat(((evaluate(*ast.left.unwrap(), vars).0 as i32) << evaluate(*ast.right.unwrap(), vars).0 as i32) as f32)
        },
        Operation::RSHIFT => {
            OrderedFloat((evaluate(*ast.left.unwrap(), vars).0 as i32 >> evaluate(*ast.right.unwrap(), vars).0 as i32) as f32)
        },
        Operation::BITAND => {
            OrderedFloat((evaluate(*ast.left.unwrap(), vars).0 as i32 & evaluate(*ast.right.unwrap(), vars).0 as i32) as f32)
        },
        Operation::BITXOR => {
            OrderedFloat((evaluate(*ast.left.unwrap(), vars).0 as i32 ^ evaluate(*ast.right.unwrap(), vars).0 as i32) as f32)
        },
        Operation::BITOR => {
            OrderedFloat((evaluate(*ast.left.unwrap(), vars).0 as i32 | evaluate(*ast.right.unwrap(), vars).0 as i32) as f32)
        },
        Operation::NEGATE => {
            -evaluate(*ast.left.unwrap(), vars)
        },
        Operation::BITNOT => {
            OrderedFloat(!(evaluate(*ast.left.unwrap(), vars).0 as i32) as f32) 
        },
        Operation::NUMBER(v) => OrderedFloat(v as f32),
        Operation::FLOAT(v) => v,
        Operation::IDENT(v) => {
            if let Some(v) = vars.get(&v) {
                *v
            } else {
                panic!("{0} is not defined", v)
            }
        }
        Operation::GLUE => {
            // aim to return the value of the last line
            evaluate(*ast.left.unwrap(), vars);
            evaluate(*ast.right.unwrap(), vars)
        } 
    }
}
