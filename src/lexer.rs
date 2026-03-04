#![allow(dead_code)] // for now until the rest is made

use peekread::BufPeekReader;
use thiserror::Error;
use utf8_chars::BufReadCharsExt;

use std::{
    fmt::{self, Display},
    io::{self, Read},
    result,
};

use peekread::PeekRead;
use std::io::Seek;

pub type Result<T> = result::Result<T, LexerError>;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct SourcePosition {
    pub line_position: usize,
    pub column_position: usize,
}

// since io::Error is not PartialEq
#[derive(Debug)]
pub struct LexerIoError(pub io::Error);

#[derive(Debug, Error, PartialEq)]
pub enum LexerError {
    #[error("Lexer Error: Unterminated String")]
    UnterminatedString(SourcePosition),

    #[error("Lexer Error: Unterminated Character Constant")]
    UnterminatedCharacterConstant(SourcePosition),

    #[error("Lexer Error: > 1 Character in Character Constant")]
    CharacterConstantTooLong(SourcePosition),

    #[error("Lexer Error: Malformed Float")]
    MalformedFloat(SourcePosition),

    #[error("Lexer Error: Unterminated Comment")]
    UnterminatedComment(SourcePosition),

    #[error("Lexer Error: Unrecognised Character")]
    UnrecognisedCharacter(SourcePosition),

    #[error("Lexer Error: IO Error: {0}")]
    Io(LexerIoError),
}

impl PartialEq for LexerIoError {
    fn eq(&self, other: &Self) -> bool {
        self.0.kind() == other.0.kind()
    }
}

impl Display for LexerIoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<io::Error> for LexerError {
    fn from(e: io::Error) -> Self {
        LexerError::Io(LexerIoError(e))
    }
}

// todo: implement Display for Tokens
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Tokens {
    // TODO: Error token
    ASSIGN,

    LOGAND,

    LOGOR,

    BITOR,
    BITXOR,
    AMPER, // called amper cause address of uses this symbol as well as bitwise AND

    EQ,
    NEQ,

    LT,
    LTE,

    GT,
    GTE,

    LSHIFT,
    RSHIFT,

    MINUS, // also unary
    PLUS,

    ASTERISK, // called asterik cause pointer uses this symbol as well as times
    DIVIDE,

    MODULO,

    NOT,
    BITNOT,

    LPARENT, // (
    RPARENT, // )

    LBRACKET, // [
    RBRACKET, // ]

    DOT,

    ARROW,

    COLON, // used for labels
    SEMICOLON,

    LBRACE,
    RBRACE,

    CHAR(char),
    STRING(String),

    NUMBER(String),
    FLOAT(String),
    IDENT(String),

    FN,
    LET,
    IF,
    ELSE,
    FOR, // no while statement, while is covered by for

    GOTO,

    STRUCT,
    UNION,

    U0, // equivalent to void
    I8,
    I16,
    I32,
    EOF,
}

// todo: keep track of lines
pub struct Lexer<R: Read> {
    input_buf: BufPeekReader<R>,
    token_putback_buf: Option<Tokens>,
    source_position: SourcePosition,
    record_position: bool,
}

impl<R: Read> Lexer<R> {
    pub fn new(input: R) -> Result<Self> {
        Ok(Self {
            input_buf: BufPeekReader::new(input),
            token_putback_buf: None,
            source_position: SourcePosition {
                line_position: 1,
                column_position: 0,
            },
            record_position: true,
        })
    }

    fn peek(&mut self, n: u64) -> Result<Option<char>> {
        let mut c = self.input_buf.peek();
        c.seek(io::SeekFrom::Start(n))?;
        Ok(c.read_char()?)
    }

    fn consume(&mut self) -> Result<Option<char>> {
        if let Some(c) = self.input_buf.read_char()? {
            if self.record_position {
                if c == '\n' {
                    self.source_position.line_position += 1;
                    self.source_position.column_position = 1;
                } else if c == '\t' {
                    self.source_position.column_position +=
                        8 - (self.source_position.column_position % 8)
                } else {
                    self.source_position.column_position += 1;
                }
            }
            Ok(Some(c))
        } else {
            Ok(None)
        }
    }

    pub fn peek_token(&mut self) -> Result<&Tokens> {
        if self.token_putback_buf.is_none() {
            self.token_putback_buf = Some(self.scan_token()?);
        }

        self.token_putback_buf
            .as_ref()
            .ok_or_else(|| unreachable!())
    }

    pub fn consume_token(&mut self) {
        self.token_putback_buf = None
    }

    pub fn take(&mut self) -> Result<Tokens> {
        match self.token_putback_buf.take() {
            Some(t) => Ok(t),
            None => self.scan_token(),
        }
    }

    fn skip_whitespace_and_comments(&mut self) -> Result<()> {
        while let Some(c) = self.peek(0)? {
            if c == '/' {
                let c2 = match self.peek(1)? {
                    Some(c) => c,
                    None => {
                        break;
                    }
                };

                if c2 == '*' {
                    self.consume()?;
                    self.consume()?;

                    loop {
                        let c = match self.peek(0)? {
                            Some(c) => c,
                            None => return Err(LexerError::UnterminatedComment(self.source_position.clone())),
                        };

                        if c == '*' {
                            let c2 = match self.peek(1)? {
                                Some(c) => c,
                                None => return Err(LexerError::UnterminatedComment(self.source_position.clone())),
                            };

                            if c2 == '/' {
                                // other character will be consumed out of loop
                                self.consume()?;
                                break;
                            }
                        }

                        self.consume()?;
                    }
                } else if c2 == '/' {
                    self.consume()?;
                    self.consume()?;

                    loop {
                        let c = match self.peek(0)? {
                            Some(c) => c,
                            None => return Ok(()),
                        };

                        if c == '\n' {
                            break;
                        }

                        self.consume()?;
                    }
                } else {
                    break;
                }
            } else if !c.is_whitespace() {
                break;
            }

            self.consume()?;
        }

        Ok(())
    }

    fn scan_token(&mut self) -> Result<Tokens> {
        self.record_position = false;
        self.skip_whitespace_and_comments()?;
        self.record_position = true;

        Ok(if let Some(c) = self.peek(0)? {
            match c {
                '&' => {
                    self.consume()?;
                    if let Some(c) = self.peek(0)? {
                        if c == '&' {
                            self.consume()?;
                            Tokens::LOGAND
                        } else {
                            Tokens::AMPER
                        }
                    } else {
                        Tokens::AMPER
                    }
                }
                '|' => {
                    self.consume()?;
                    if let Some(c) = self.peek(0)? {
                        if c == '|' {
                            self.consume()?;
                            Tokens::LOGOR
                        } else {
                            Tokens::BITOR
                        }
                    } else {
                        Tokens::BITOR
                    }
                }
                '=' => {
                    self.consume()?;
                    if let Some(c) = self.peek(0)? {
                        if c == '=' {
                            self.consume()?;
                            Tokens::EQ
                        } else {
                            Tokens::ASSIGN
                        }
                    } else {
                        Tokens::ASSIGN
                    }
                }

                '!' => {
                    self.consume()?;
                    if let Some(c) = self.peek(0)? {
                        if c == '=' {
                            self.consume()?;
                            Tokens::NEQ
                        } else {
                            Tokens::NOT
                        }
                    } else {
                        Tokens::NOT
                    }
                }

                '<' => {
                    self.consume()?;
                    if let Some(c) = self.peek(0)? {
                        if c == '=' {
                            self.consume()?;
                            Tokens::LTE
                        } else if c == '<' {
                            self.consume()?;
                            Tokens::LSHIFT
                        } else {
                            Tokens::LT
                        }
                    } else {
                        Tokens::LT
                    }
                }

                '>' => {
                    self.consume()?;
                    if let Some(c) = self.peek(0)? {
                        if c == '=' {
                            self.consume()?;
                            Tokens::GTE
                        } else if c == '>' {
                            self.consume()?;
                            Tokens::RSHIFT
                        } else {
                            Tokens::GT
                        }
                    } else {
                        Tokens::GT
                    }
                }

                '^' => {
                    self.consume()?;
                    Tokens::BITXOR
                }
                '-' => {
                    self.consume()?;
                    if let Some(c) = self.peek(0)? {
                        if c == '>' {
                            self.consume()?;
                            Tokens::ARROW
                        } else {
                            Tokens::MINUS
                        }
                    } else {
                        Tokens::MINUS
                    }
                }
                '+' => {
                    self.consume()?;
                    Tokens::PLUS
                }
                '*' => {
                    self.consume()?;
                    Tokens::ASTERISK
                }
                '/' => {
                    self.consume()?;
                    Tokens::DIVIDE
                }
                '%' => {
                    self.consume()?;
                    Tokens::MODULO
                }
                '~' => {
                    self.consume()?;
                    Tokens::BITNOT
                }
                '(' => {
                    self.consume()?;
                    Tokens::LPARENT
                }
                ')' => {
                    self.consume()?;
                    Tokens::RPARENT
                }
                '[' => {
                    self.consume()?;
                    Tokens::LBRACKET
                }
                ']' => {
                    self.consume()?;
                    Tokens::RBRACKET
                }
                '{' => {
                    self.consume()?;
                    Tokens::LBRACE
                }
                '}' => {
                    self.consume()?;
                    Tokens::RBRACE
                }
                '.' => {
                    if let Some(c) = self.peek(0)? {
                        if c.is_digit(10) {
                            return self.scan_float();
                        } else {
                            Tokens::DOT
                        }
                    } else {
                        Tokens::DOT
                    }
                }
                ':' => {
                    self.consume()?;
                    Tokens::COLON
                }
                ';' => {
                    self.consume()?;
                    Tokens::SEMICOLON
                }
                '\'' => {
                    self.consume()?;
                    let chr = self.scan_char()?;
                    Tokens::CHAR(chr)
                }
                '"' => {
                    self.consume()?;
                    let str = self.scan_string()?;
                    Tokens::STRING(str)
                }
                _ => {
                    if c.is_numeric() {
                        self.scan_number()?
                    } else if c.is_alphabetic() || c == '_' {
                        self.match_keyword()?
                    } else {
                        return Err(LexerError::UnrecognisedCharacter(self.source_position.clone()));
                    }
                }
            }
        } else {
            Tokens::EOF
        })
    }

    // NEGATIVE NUMBERS TO BE HANDLED BY PARSER
    fn scan_number(&mut self) -> Result<Tokens> {
        // safe to unwrap twice since we know its real before
        let c = self.peek(0).unwrap().unwrap();

        if c == '0' {
            self.consume()?;
            if let Some(c) = self.peek(0)? {
                if c == 'x' {
                    self.consume()?;
                    Ok(Tokens::NUMBER("0x".to_string() + &self.scan_base(16)?))
                } else {
                    Ok(Tokens::NUMBER("0".to_string() + &self.scan_base(8)?))
                }
            } else {
                self.scan_float()
            }
        } else {
            self.scan_float()
        }
    }

    fn scan_base(&mut self, base: u32) -> Result<String> {
        let mut num = String::new();
        while let Some(c) = self.peek(0)? {
            if c.is_digit(base) {
                // parsers responsibility to throw int overflow error
                self.consume()?;
                num.push(c);
            } else {
                break;
            }
        }

        Ok(num)
    }

    fn scan_float(&mut self) -> Result<Tokens> {
        let mut num = self.scan_base(10)?;

        if let Some(c) = self.peek(0)? {
            if c == '.' {
                self.consume()?;
                num.push('.');

                while let Some(c) = self.peek(0)? {
                    if c.is_digit(10) {
                        self.consume()?;
                        num.push(c);
                    } else {
                        break;
                    }
                }

                self.read_exponent(&mut num)?;

                Ok(Tokens::FLOAT(num))
            } else {
                let is_float = self.read_exponent(&mut num)?;

                Ok(if is_float {
                    Tokens::FLOAT(num)
                } else {
                    Tokens::NUMBER(num)
                })
            }
        } else {
            Ok(Tokens::NUMBER(num))
        }
    }

    fn read_exponent(&mut self, num: &mut String) -> Result<bool> {
        if let Some(c) = self.peek(0)? {
            if c == 'e' || c == 'E' {
                self.consume()?;
                num.push(c);
                if let Some(c) = self.peek(0)? {
                    if c == '-' || c == '+' {
                        self.consume()?;
                        num.push(c);
                    } else if !c.is_digit(10) {
                        // ended on E
                        return Err(LexerError::MalformedFloat(self.source_position.clone()));
                    }
                } else {
                    // EOF reached on ended E
                    return Err(LexerError::MalformedFloat(self.source_position.clone()));
                }

                num.push_str(&self.scan_base(10)?);
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    fn match_keyword(&mut self) -> Result<Tokens> {
        let ident = self.scan_ident()?;
        match ident.as_str() {
            "fn" => Ok(Tokens::FN),
            "let" => Ok(Tokens::LET),
            "if" => Ok(Tokens::IF),
            "else" => Ok(Tokens::ELSE),
            "for" => Ok(Tokens::FOR),
            "goto" => Ok(Tokens::GOTO),
            "struct" => Ok(Tokens::STRUCT),
            "union" => Ok(Tokens::UNION),
            "u0" => Ok(Tokens::U0),
            "i8" => Ok(Tokens::I8),
            "i16" => Ok(Tokens::I16),
            "i32" => Ok(Tokens::I32),
            _ => Ok(Tokens::IDENT(ident)),
        }
    }

    fn scan_ident(&mut self) -> Result<String> {
        let mut ident = String::new();

        while let Some(c) = self.peek(0)? {
            if !(c.is_alphanumeric() || c == '_') {
                break;
            }

            self.consume()?;
            ident.push(c);
        }

        Ok(ident)
    }

    fn scan_string(&mut self) -> Result<String> {
        let mut str = String::new();

        while let Some(c) = self.peek(0)? {
            if c == '\n' {
                return Err(LexerError::UnterminatedString(self.source_position.clone()));
            }

            self.consume()?;

            if c == '\"' {
                return Ok(str);
            }

            if c == '\\' {
                if let Some(escape_chr) = self.peek(0)? {
                    match escape_chr {
                        'n' => str.push('\n'),
                        'r' => str.push('\r'),
                        't' => str.push('\t'),
                        '"' => str.push('\"'),
                        _ => {
                            str.push('\\');
                            continue;
                        }
                    }

                    self.consume()?;
                    continue;
                }
            }

            str.push(c);
        }

        Err(LexerError::UnterminatedString(self.source_position.clone()))
    }

    fn scan_char(&mut self) -> Result<char> {
        if let Some(c) = self.peek(0)? {
            self.consume()?;
            if let Some(c2) = self.peek(0)? {
                if c2 == '\'' {
                    self.consume()?;
                    return Ok(c);
                } else {
                    return Err(LexerError::CharacterConstantTooLong(self.source_position.clone()));
                }
            } else {
                return Err(LexerError::UnterminatedCharacterConstant(self.source_position.clone()));
            }
        } else {
            return Err(LexerError::UnterminatedCharacterConstant(self.source_position.clone()));
        }
    }
}
