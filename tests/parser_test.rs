#[cfg(test)]
mod tests {
    use madx_language::lexer::{Lexer, LexerError};
    use madx_language::parser::{AST, Operation, Parser, ParserError};

    macro_rules! ast_tree {
        {$op: expr, { $($left : tt)* }, { $($right : tt)* } } => {
            $crate::tests::AST{
                op: $op,
                left: Some(Box::new(ast_tree!( $($left)* ))),
                right: Some(Box::new(ast_tree!( $($right)*)))
            }
        };
        {$op: expr, { $($left: tt)* }} => {
            $crate::tests::AST{
                op: $op,
                left: Some(Box::new(ast_tree!($($left)*))),
                right: None
            }
        };

        {$op: expr} => {
            $crate::tests::AST{
                op : $op,
                left : None,
                right : None
            }
        };
    }

    #[test]
    fn parses_basic_expression() -> Result<(), ParserError> {
        let input = "1 + 2 / 3 - 4 * 5 % 6;";

        let lexer = Lexer::new(input.as_bytes())?;
        let mut parser = Parser::new(lexer);

        /*
                 -
               /   \
            /        \
          +           %
         / \        /  \
        1  '/'     *    6
          /  \    / \
         2    3  4   5



         */
        let tree = parser.parse()?;
        assert!(tree.is_some());
        vec![1, 2];
        let tree = tree.unwrap();
        assert_eq!(
            tree,
            ast_tree! {
                   Operation::SUBTRACT,
                       { Operation::ADD,
                       {
                           Operation::NUMBER(1)
                       },
                       { Operation::DIVIDE, {
                           Operation::NUMBER(2)
                       }, {
                           Operation::NUMBER(3)
                       } }
                   }, { 
                    Operation::MODULO,
                    {
                        Operation::MULTIPLY,
                        {
                            Operation::NUMBER(4)
                        },
                        {
                            Operation::NUMBER(5)
                        }
                    }, {
                        Operation::NUMBER(6)
                    }
                 }
            }
        );

        Ok(())
    }
}
