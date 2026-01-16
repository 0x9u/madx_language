use std::{io::Read, result};

use thiserror::Error;

use crate::lexer::{Lexer, LexerError, Tokens};

pub type Result<T> = result::Result<T, ParserError>;

#[derive(Debug, Error, PartialEq)]
pub enum ParserError {
    #[error("Parser Error: expected {0:?}, got {1:?}")]
    ExpectError(Tokens, Tokens),

    #[error("Parser Error: syntax error")]
    SyntaxError,

    #[error("{0}")]
    Lexer(LexerError),
}

impl From<LexerError> for ParserError {
    fn from(e: LexerError) -> Self {
        ParserError::Lexer(e)
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum Operation {
    ADD,
    SUBTRACT,
    MULTIPLY,
    DIVIDE,
    MODULO,
    ASSIGN,
    NUMBER(i32),
    IDENT(String),
}

pub struct AST {
    pub op: Operation,
    pub left: Option<Box<AST>>,
    pub right: Option<Box<AST>>,
}

pub struct Parser<R: Read> {
    lexer: Lexer<R>,
}

impl<R: Read> Parser<R> {
    pub fn new(lexer: Lexer<R>) -> Self {
        Self { lexer }
    }

    fn expect(&mut self, t: Tokens) -> Result<()> {
        let t2 = self.lexer.take()?;
        if t2 != t {
            Result::Err(ParserError::ExpectError(t, t2))
        } else {
            Result::Ok(())
        }
    }

    fn factor(&mut self) -> Result<AST> {
        Result::Ok(AST {
            op: {
                // todo: make this into a macro
                let v = match self.lexer.peek()?.clone() {
                    Tokens::NUMBER(v) => Operation::NUMBER(v),
                    Tokens::IDENT(v) => Operation::IDENT(v),
                    _ => return Result::Err(ParserError::SyntaxError),
                };
                self.lexer.consume();
                v
            },
            left: None,
            right: None,
        })
    }

    fn mult(&mut self) -> Result<AST> {
        let mut left = self.factor()?;

        loop {
            left = AST {
                op: {
                    let v = match self.lexer.peek()?.clone() {
                        Tokens::ASTERISK => Operation::MULTIPLY,
                        Tokens::DIVIDE => Operation::DIVIDE,
                        _ => {
                            break Result::Ok(left);
                        }
                    };
                    self.lexer.consume();
                    v
                },
                left: Some(Box::new(left)),
                right: Some(Box::new(self.factor()?)),
            }
        }
    }

    fn add(&mut self) -> Result<AST> {
        let mut left = self.mult()?;

        loop {
            left = AST {
                op: {
                    let v = match self.lexer.peek()?.clone() {
                        Tokens::PLUS => Operation::ADD,
                        Tokens::MINUS => Operation::SUBTRACT,
                        _ => {
                            break Result::Ok(left);
                        }
                    };
                    self.lexer.consume();
                    v
                },
                left: Some(Box::new(left)),
                right: Some(Box::new(self.mult()?)),
            }
        }
    }

    fn assign(&mut self) -> Result<AST> {
        let mut left = self.add()?;

        while matches!(left.op, Operation::IDENT(_)) {
            left = AST {
                op: if self.lexer.peek()?.clone() == Tokens::ASSIGN {
                    self.lexer.consume();
                    Operation::ASSIGN
                } else {
                    return Result::Ok(left);
                },
                left: Some(Box::new(left)),
                right: Some(Box::new(self.add()?)),
            }
        }

        Ok(left)
    }

    // should be block but whatever
    fn statement(&mut self) -> Result<Vec<AST>> {

        let mut exprs = Vec::new();

        while self.lexer.peek()? != &Tokens::EOF {
            exprs.push(self.assign()?);
            self.expect(Tokens::SEMICOLON)?;
        }

        Result::Ok(exprs)
    }

    pub fn parse(&mut self) -> Result<Vec<AST>> {
        self.statement()
    }
}
