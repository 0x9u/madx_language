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
pub enum Operation { // todo: define left and right AST enum operations
    NUMBER(i32),
    IDENT(String),

    BITNOT,
    NEGATE,

    MULTIPLY,
    DIVIDE,
    MODULO,
    ADD,
    SUBTRACT,

    LSHIFT,
    RSHIFT,

    BITOR,
    BITXOR,
    BITAND,

    ASSIGN,
    GLUE, // to join statements together
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
        let t = self.lexer.peek()?;

        if t == &Tokens::LPARENT {
            self.lexer.consume();
            let tree = self.assign()?;
            self.expect(Tokens::RPARENT)?;
            Result::Ok(tree)
        } else {
            Result::Ok(AST {
                op: {
                    // todo: make this into a macro
                    let v = match t {
                        Tokens::NUMBER(v) => Operation::NUMBER(*v),
                        Tokens::IDENT(v) => Operation::IDENT(v.clone()),
                        _ => return Result::Err(ParserError::SyntaxError),
                    };
                    self.lexer.consume();
                    v
                },
                left: None,
                right: None,
            })
        }
    }

    fn unary(&mut self) -> Result<AST> {
        match *self.lexer.peek()? {
            Tokens::PLUS => {
                self.lexer.consume();
                self.unary()
            } // literally useless
            Tokens::MINUS => {
                self.lexer.consume();
                Result::Ok(AST {
                    op: Operation::NEGATE,
                    left: Some(Box::new(self.unary()?)),
                    right: None,
                })
            }
            Tokens::BITNOT => {
                self.lexer.consume();
                Result::Ok(AST {
                    op: Operation::BITNOT,
                    left: Some(Box::new(self.unary()?)),
                    right: None,
                })
            }
            _ => self.factor(),
        }
    }

    fn mult(&mut self) -> Result<AST> {
        let mut left = self.unary()?;

        loop {
            left = AST {
                op: {
                    let v = match *self.lexer.peek()? {
                        Tokens::ASTERISK => Operation::MULTIPLY,
                        Tokens::DIVIDE => Operation::DIVIDE,
                        Tokens::MODULO => Operation::MODULO,
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
                    let v = match *self.lexer.peek()? {
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

    fn bitwise_shift(&mut self) -> Result<AST> {
        let mut left = self.add()?;

        loop {
            left = AST {
                op: {
                    let v = match *self.lexer.peek()? {
                        Tokens::LSHIFT => Operation::LSHIFT,
                        Tokens::RSHIFT => Operation::RSHIFT,
                        _ => {
                            break Result::Ok(left);
                        }
                    };
                    self.lexer.consume();
                    v
                },
                left: Some(Box::new(left)),
                right: Some(Box::new(self.add()?)),
            }
        }
    }

    fn bitwise_and(&mut self) -> Result<AST> {
        let mut left = self.bitwise_shift()?;

        loop {
            left = AST {
                op: if self.lexer.peek()? == &Tokens::AMPER {
                    self.lexer.consume();
                    Operation::BITAND
                } else {
                    break Result::Ok(left);
                },
                left: Some(Box::new(left)),
                right: Some(Box::new(self.bitwise_shift()?)),
            }
        }
    }

    fn bitwise_xor(&mut self) -> Result<AST> {
        let mut left = self.bitwise_and()?;

        loop {
            left = AST {
                op: if self.lexer.peek()? == &Tokens::BITXOR {
                    self.lexer.consume();
                    Operation::BITXOR
                } else {
                    break Result::Ok(left);
                },
                left: Some(Box::new(left)),
                right: Some(Box::new(self.bitwise_and()?)),
            }
        }
    }

    fn bitwise_or(&mut self) -> Result<AST> {
        let mut left = self.bitwise_xor()?;

        loop {
            left = AST {
                op: if self.lexer.peek()? == &Tokens::BITOR {
                    self.lexer.consume();
                    Operation::BITOR
                } else {
                    break Result::Ok(left);
                },
                left: Some(Box::new(left)),
                right: Some(Box::new(self.bitwise_xor()?)),
            }
        }
    }

    fn assign(&mut self) -> Result<AST> {
        let left = self.bitwise_or()?;

        // todo: figure out recursion
        if *self.lexer.peek()? == Tokens::ASSIGN {
            self.lexer.consume();
            Result::Ok(AST {
                op: Operation::ASSIGN,
                left: Some(Box::new(left)),
                right: Some(Box::new(self.assign()?)),
            })
        } else {
            Result::Ok(left)
        }
    }

    fn statement(&mut self) -> Result<Option<AST>> {
        let t = self.lexer.peek()?;

        if t == &Tokens::LBRACE {
            self.compound_statement()
        } else {
            // expression statement is optional
            if t == &Tokens::SEMICOLON {
                self.lexer.consume();
                return Result::Ok(None);
            }

            let tree = self.assign()?;
            self.expect(Tokens::SEMICOLON)?;
            Result::Ok(Some(tree))
        }
    }

    fn compound_statement(&mut self) -> Result<Option<AST>> {
        self.expect(Tokens::LBRACE)?;

        let mut parent: Option<AST> = None;

        while self.lexer.peek()? != &Tokens::RBRACE {
            if let Some(right) = self.statement()? {
                if let Some(left) = parent {
                    parent = Some(AST {
                        op: Operation::GLUE,
                        left: Some(Box::new(left)),
                        right: Some(Box::new(right)),
                    });
                } else {
                    parent = Some(right);
                }
            }
        }

        self.expect(Tokens::RBRACE)?;

        Result::Ok(parent)
    }

    pub fn parse(&mut self) -> Result<Option<AST>> {
        self.statement()
    }
}
