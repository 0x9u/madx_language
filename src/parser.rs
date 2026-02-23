use std::{io::Read, num::{ParseFloatError, ParseIntError}, result};

use ordered_float::OrderedFloat;
use thiserror::Error;

use crate::lexer::{Lexer, LexerError, Tokens};

pub type Result<T> = result::Result<T, ParserError>;

#[derive(Debug, Error, PartialEq)]
pub enum ParserError {
    #[error("Parser Error: expected {0:?}, got {1:?}")]
    ExpectError(Tokens, Tokens),

    #[error("Parser Error: syntax error")]
    SyntaxError,

    #[error("Parser Error: Could not convert to int: {0}")]
    ParseIntError(ParseIntError),

    #[error("Parser Error: Could not convert to float: {0}")]
    ParseFloatError(ParseFloatError),

    #[error("{0}")]
    Lexer(LexerError),
}

impl From<LexerError> for ParserError {
    fn from(e: LexerError) -> Self {
        ParserError::Lexer(e)
    }
}

impl From<ParseIntError> for ParserError {
    fn from(e: ParseIntError) -> Self {
        ParserError::ParseIntError(e)
    }
}

impl From<ParseFloatError> for ParserError {
    fn from(e: ParseFloatError) -> Self {
        ParserError::ParseFloatError(e)
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum Operation {
    // todo: define left and right AST enum operations
    NUMBER(i32),
    FLOAT(OrderedFloat<f32>),
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
                    let v = match t {
                        Tokens::NUMBER(v) => Operation::NUMBER(if v.starts_with("0x") {
                            i32::from_str_radix(v.trim_start_matches("0x"), 16)?
                        } else if v.starts_with("0") {
                            let mut trim = v.chars();
                            trim.next();
                            i32::from_str_radix(trim.as_str(), 8)?
                        } else {
                            i32::from_str_radix(v, 10)?
                        }),
                        Tokens::FLOAT(v) => 
                            Operation::FLOAT(OrderedFloat(v.parse()?)),
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
        self.binary_op(Self::unary, |t| match t {
            Tokens::ASTERISK => Some(Operation::MULTIPLY),
            Tokens::DIVIDE => Some(Operation::DIVIDE),
            Tokens::MODULO => Some(Operation::MODULO),
            _ => None,
        })
    }

    fn add(&mut self) -> Result<AST> {
        self.binary_op(Self::mult, |t| match t {
            Tokens::PLUS => Some(Operation::ADD),
            Tokens::MINUS => Some(Operation::SUBTRACT),
            _ => None,
        })
    }

    fn bitwise_shift(&mut self) -> Result<AST> {
        self.binary_op(Self::add, |t| match t {
            Tokens::LSHIFT => Some(Operation::LSHIFT),
            Tokens::RSHIFT => Some(Operation::RSHIFT),
            _ => None,
        })
    }

    fn bitwise_and(&mut self) -> Result<AST> {
        self.binary_op(Self::bitwise_shift, |t| {
            if t == &Tokens::AMPER {
                Some(Operation::BITAND)
            } else {
                None
            }
        })
    }

    fn bitwise_xor(&mut self) -> Result<AST> {
        self.binary_op(Self::bitwise_and, |t| {
            if t == &Tokens::BITXOR {
                Some(Operation::BITXOR)
            } else {
                None
            }
        })
    }

    fn bitwise_or(&mut self) -> Result<AST> {
        self.binary_op(Self::bitwise_xor, |t| {
            if t == &Tokens::BITOR {
                Some(Operation::BITOR)
            } else {
                None
            }
        })
    }

    // matcher uses the Fn trait to account for closures, used to match tokens to operations
    // next_level takes a method from Self, which is used to parse operators of higher precedence
    fn binary_op<F, S>(&mut self, next_level: S, matcher: F) -> Result<AST>
    where
        F: Fn(&Tokens) -> Option<Operation>,
        for<'a> S: Fn(&'a mut Self) -> Result<AST>,
    {
        let mut left = next_level(self)?;

        loop {
            left = AST {
                op: {
                    let v = match matcher(self.lexer.peek()?) {
                        Some(v) => v,
                        None => break Result::Ok(left),
                    };
                    self.lexer.consume();
                    v
                },
                left: Some(Box::new(left)),
                right: Some(Box::new(next_level(self)?)),
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
