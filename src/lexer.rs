#![allow(dead_code)] // for now until the rest is made

use thiserror::Error;

use ordered_float::OrderedFloat;
use std::{
    collections::VecDeque,
    fmt::{self, Display},
    io::{self, BufReader, Read},
    result,
};
use utf8_chars::BufReadCharsExt;

pub type Result<T> = result::Result<T, LexerError>;

// since io::Error is not PartialEq
#[derive(Debug)]
pub struct LexerIoError(pub io::Error);

#[derive(Debug, Error, PartialEq)]
pub enum LexerError {
    #[error("Lexer Error: Unterminated String")]
    UnterminatedString,

    #[error("Lexer Error: Unterminated Character Constant")]
    UnterminatedCharacterConstant,

    #[error("Lexer Error: > 1 Character in Character Constant")]
    CharacterConstantTooLong,

    #[error("Lexer Error: Integer Overflow")]
    IntegerOverflow,

    #[error("Lexer Error: Malformed Float")]
    MalformedFloat,

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

    NUMBER(i32),
    FLOAT(OrderedFloat<f32>),
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
    input_buf: BufReader<R>,
    char_putback_buf: VecDeque<char>,
    token_putback_buf: Option<Tokens>,
}

impl<R: Read> Lexer<R> {
    pub fn new(input: R) -> Result<Self> {
        Ok(Self {
            input_buf: BufReader::new(input),
            char_putback_buf: VecDeque::new(),
            token_putback_buf: None,
        })
    }

    fn read(&mut self) -> Result<Option<char>> {
        if let Some(c) = self.char_putback_buf.pop_front() {
            Result::Ok(Some(c))
        } else if let Some(c) = self.input_buf.read_char()? {
            Result::Ok(Some(c))
        } else {
            Result::Ok(None)
        }
    }

    fn putback(&mut self, c: char) {
        self.char_putback_buf.push_back(c);
    }

    pub fn peek(&mut self) -> Result<&Tokens> {
        if self.token_putback_buf.is_none() {
            self.token_putback_buf = Some(self.scan_token()?);
        }

        self.token_putback_buf
            .as_ref()
            .ok_or_else(|| unreachable!())
    }

    pub fn consume(&mut self) {
        self.token_putback_buf = None
    }

    pub fn take(&mut self) -> Result<Tokens> {
        match self.token_putback_buf.take() {
            Some(t) => Ok(t),
            None => self.scan_token(),
        }
    }

    fn scan_token(&mut self) -> Result<Tokens> {
        while let Some(c) = self.read()? {
            // TODO: skip comments (// and /* */)
            if c.is_whitespace() {
                continue;
            }

            match c {
                '&' => {
                    if let Some(c) = self.read()? {
                        if c == '&' {
                            return Result::Ok(Tokens::LOGAND);
                        } else {
                            self.putback(c);
                            return Result::Ok(Tokens::AMPER);
                        }
                    } else {
                        return Result::Ok(Tokens::AMPER);
                    }
                }
                '|' => {
                    if let Some(c) = self.read()? {
                        if c == '|' {
                            return Result::Ok(Tokens::LOGOR);
                        } else {
                            self.putback(c);
                            return Result::Ok(Tokens::BITOR);
                        }
                    } else {
                        return Result::Ok(Tokens::BITOR);
                    }
                }
                '=' => {
                    if let Some(c) = self.read()? {
                        if c == '=' {
                            return Result::Ok(Tokens::EQ);
                        } else {
                            self.putback(c);
                            return Result::Ok(Tokens::ASSIGN);
                        }
                    } else {
                        return Result::Ok(Tokens::ASSIGN);
                    }
                }

                '!' => {
                    if let Some(c) = self.read()? {
                        if c == '=' {
                            return Result::Ok(Tokens::NEQ);
                        } else {
                            self.putback(c);
                            return Result::Ok(Tokens::NOT);
                        }
                    } else {
                        return Result::Ok(Tokens::NOT);
                    }
                }

                '<' => {
                    if let Some(c) = self.read()? {
                        if c == '=' {
                            return Result::Ok(Tokens::LTE);
                        } else if c == '<' {
                            return Result::Ok(Tokens::LSHIFT);
                        } else {
                            self.putback(c);
                            return Result::Ok(Tokens::LT);
                        }
                    } else {
                        return Result::Ok(Tokens::LT);
                    }
                }

                '>' => {
                    if let Some(c) = self.read()? {
                        if c == '=' {
                            return Result::Ok(Tokens::GTE);
                        } else if c == '>' {
                            return Result::Ok(Tokens::RSHIFT);
                        } else {
                            self.putback(c);
                            return Result::Ok(Tokens::GT);
                        }
                    } else {
                        return Result::Ok(Tokens::GT);
                    }
                }

                '^' => return Result::Ok(Tokens::BITXOR),
                '-' => {
                    if let Some(c) = self.read()? {
                        if c == '>' {
                            return Result::Ok(Tokens::ARROW);
                        } else {
                            self.putback(c);
                            return Result::Ok(Tokens::MINUS);
                        }
                    } else {
                        return Result::Ok(Tokens::MINUS);
                    }
                }
                '+' => return Result::Ok(Tokens::PLUS),
                '*' => return Result::Ok(Tokens::ASTERISK),
                '/' => return Result::Ok(Tokens::DIVIDE),
                '%' => return Result::Ok(Tokens::MODULO),
                '~' => return Result::Ok(Tokens::BITNOT),
                '(' => return Result::Ok(Tokens::LPARENT),
                ')' => return Result::Ok(Tokens::RPARENT),
                '[' => return Result::Ok(Tokens::LBRACKET),
                ']' => return Result::Ok(Tokens::RBRACKET),
                '{' => return Result::Ok(Tokens::LBRACE),
                '}' => return Result::Ok(Tokens::RBRACE),
                '.' => {
                    if let Some(c) = self.read()? {
                        if c.is_digit(10) {
                            return self.scan_float();
                        } else {
                            self.putback(c);
                            return Result::Ok(Tokens::DOT);
                        }
                    } else {
                        return Result::Ok(Tokens::DOT);
                    }
                }
                ':' => return Result::Ok(Tokens::COLON),
                ';' => return Result::Ok(Tokens::SEMICOLON),
                '\'' => {
                    let chr = self.scan_char()?;
                    return Result::Ok(Tokens::CHAR(chr));
                }
                '"' => {
                    let str = self.scan_string()?;
                    return Result::Ok(Tokens::STRING(str));
                }
                _ => {
                    if c.is_numeric() {
                        self.putback(c); // to be consumed by scan_number
                        return self.scan_number();
                    } else {
                        self.putback(c); // to be consumed by keyword
                        return self.match_keyword();
                    }
                }
            }
        }

        Result::Ok(Tokens::EOF)
    }

    // NEGATIVE NUMBERS TO BE HANDLED BY PARSER
    fn scan_number(&mut self) -> Result<Tokens> {
        // safe to unwrap twice since we know we putback
        let c = self.read().unwrap().unwrap();

        if c == '0' {
            if let Some(c) = self.read()? {
                // todo: avoid redundant wrapping

                if c == 'x' {
                    Result::Ok(Tokens::NUMBER(self.scan_base(16)?))
                } else {
                    self.putback(c);
                    Result::Ok(Tokens::NUMBER(self.scan_base(8)?))
                }
            } else {
                self.putback(c);
                self.scan_float()
            }
        } else {
            self.putback(c);
            self.scan_float()
        }
    }

    fn scan_base(&mut self, base: u32) -> Result<i32> {
        let mut num = 0_i32;
        while let Some(c) = self.read()? {
            if let Some(d) = c.to_digit(base) {
                num *= base as i32;
                num = match num.checked_add(d as i32) {
                    Some(n) => n,
                    None => return Result::Err(LexerError::IntegerOverflow),
                }
            } else {
                self.putback(c);
                break;
            }
        }

        Ok(num)
    }

    fn scan_float(&mut self) -> Result<Tokens> {
        let front = self.scan_base(10)? as f32;

        if let Some(c) = self.read()? {
            if c == '.' {
                let mut mantissa = 0_f32;
                let mut position = 1_f32;
                while let Some(c) = self.read()? {
                    if let Some(d) = c.to_digit(10) {
                        position /= 10.0;
                        mantissa += d as f32 * position;
                    } else {
                        self.putback(c);
                        break;
                    }
                }
                let float = mantissa + front;
                Result::Ok(Tokens::FLOAT(OrderedFloat(self.read_exponent(float)?)))
            } else {
                self.putback(c);
                Result::Ok(Tokens::NUMBER(self.read_exponent(front)? as i32))
            }
        } else {
            Result::Ok(Tokens::NUMBER(front as i32))
        }
    }

    fn read_exponent(&mut self, num: f32) -> Result<f32> {
        if let Some(c) = self.read()? {
            if c == 'e' || c == 'E' {
                let is_frac = {
                    if let Some(c) = self.read()? {
                        if c == '-' {
                            Result::Ok(true)
                        } else if c != '+' {
                            // + is the most useless aaah character
                            self.putback(c);
                            Result::Ok(false)
                        } else {
                            // still need to consume that +
                            Result::Ok(false)
                        }
                    } else {
                        // EOF reached on ended E
                        Result::Err(LexerError::MalformedFloat)
                    }
                }?;

                let exp = self.scan_base(10)?;

                let mult = 10_f32;
                Result::Ok(
                    num * mult.powf(
                        exp as f32
                            * if is_frac { -1_f32 } else { 1_f32 },
                    ),
                )
            } else {
                self.putback(c);
                Result::Ok(num)
            }
        } else {
            Result::Ok(num)
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

        while let Some(c) = self.read()? {
            if !c.is_alphanumeric() {
                self.putback(c);
                break;
            }

            ident.push(c);
        }

        Result::Ok(ident)
    }

    fn scan_string(&mut self) -> Result<String> {
        let mut str = String::new();

        while let Some(c) = self.read()? {
            if c == '\"' {
                return Result::Ok(str);
            }

            if c == '\\' {
                if let Some(escape_chr) = self.read()? {
                    match escape_chr {
                        'n' => str.push('\n'),
                        'r' => str.push('\r'),
                        't' => str.push('\t'),
                        '"' => str.push('\"'),
                        _ => {
                            self.putback(escape_chr);
                            str.push('\\');
                        }
                    }
                    continue;
                }
            }

            str.push(c);
        }

        Result::Err(LexerError::UnterminatedString)
    }

    fn scan_char(&mut self) -> Result<char> {
        // ? NOTE: I didn't putback on error since string doesn't do that as well

        if let Some(c) = self.read()? {
            if let Some(c2) = self.read()? {
                if c2 == '\'' {
                    return Result::Ok(c);
                } else {
                    return Result::Err(LexerError::CharacterConstantTooLong);
                }
            } else {
                return Result::Err(LexerError::UnterminatedCharacterConstant);
            }
        } else {
            return Result::Err(LexerError::UnterminatedCharacterConstant);
        }
    }
}
