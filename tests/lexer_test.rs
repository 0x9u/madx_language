
#[cfg(test)]
mod tests {
    use madx_language::lexer::{Lexer, LexerError, Tokens};

    #[test]
    fn scans_long_number() -> Result<(), LexerError> {
        let input = "123456789";

        let mut lexer = Lexer::new(input.as_bytes())?;

        assert_eq!(lexer.take()?, Tokens::NUMBER(123456789));

        assert_eq!(lexer.take()?, Tokens::EOF);

        Ok(())
    }

    #[test]
    fn scans_math_expression() -> Result<(), LexerError> {
        
        let input = "1 + 1 * 2 / 4 - 9 % 5";

        let mut lexer = Lexer::new(input.as_bytes())?;

        assert_eq!(lexer.take()?, Tokens::NUMBER(1));

        assert_eq!(lexer.take()?, Tokens::PLUS);

        assert_eq!(lexer.take()?, Tokens::NUMBER(1));

        assert_eq!(lexer.take()?, Tokens::ASTERISK);

        assert_eq!(lexer.take()?, Tokens::NUMBER(2));

        assert_eq!(lexer.take()?, Tokens::DIVIDE);

        assert_eq!(lexer.take()?, Tokens::NUMBER(4));

        assert_eq!(lexer.take()?, Tokens::MINUS);

        assert_eq!(lexer.take()?, Tokens::NUMBER(9));

        assert_eq!(lexer.take()?, Tokens::MODULO);

        assert_eq!(lexer.take()?, Tokens::NUMBER(5));

        assert_eq!(lexer.take()?, Tokens::EOF);

        Ok(())
    }

    #[test]
    fn scans_boolean_expression() -> Result<(), LexerError> {
        let input = "1 && 2 || 2 <= 6 < 7 >= 3 > 4 == 8 != 9";

        let mut lexer = Lexer::new(input.as_bytes())?;

        assert_eq!(lexer.take()?, Tokens::NUMBER(1));

        assert_eq!(lexer.take()?, Tokens::LOGAND);

        assert_eq!(lexer.take()?, Tokens::NUMBER(2));

        assert_eq!(lexer.take()?, Tokens::LOGOR);

        assert_eq!(lexer.take()?, Tokens::NUMBER(2));

        assert_eq!(lexer.take()?, Tokens::LTE);

        assert_eq!(lexer.take()?, Tokens::NUMBER(6));

        assert_eq!(lexer.take()?, Tokens::LT);

        assert_eq!(lexer.take()?, Tokens::NUMBER(7));

        assert_eq!(lexer.take()?, Tokens::GTE);

        assert_eq!(lexer.take()?, Tokens::NUMBER(3));

        assert_eq!(lexer.take()?, Tokens::GT);

        assert_eq!(lexer.take()?, Tokens::NUMBER(4));

        assert_eq!(lexer.take()?, Tokens::EQ);

        assert_eq!(lexer.take()?, Tokens::NUMBER(8));

        assert_eq!(lexer.take()?, Tokens::NEQ);

        assert_eq!(lexer.take()?, Tokens::NUMBER(9));

        assert_eq!(lexer.take()?, Tokens::EOF);

        Ok(())
    }

    #[test]
    fn scans_bitwise_operators() -> Result<(), LexerError> {
        let input = "1 & 2 | 3 ^ 4";

        let mut lexer = Lexer::new(input.as_bytes())?;

        assert_eq!(lexer.take()?, Tokens::NUMBER(1));

        assert_eq!(lexer.take()?, Tokens::AMPER);

        assert_eq!(lexer.take()?, Tokens::NUMBER(2));

        assert_eq!(lexer.take()?, Tokens::BITOR);

        assert_eq!(lexer.take()?, Tokens::NUMBER(3));

        assert_eq!(lexer.take()?, Tokens::BITXOR);

        assert_eq!(lexer.take()?, Tokens::NUMBER(4));

        assert_eq!(lexer.take()?, Tokens::EOF);

        Ok(())
    }

    #[test]
    fn scans_assignment() -> Result<(), LexerError> {
        let input = "a = b";

        let mut lexer = Lexer::new(input.as_bytes())?;

        assert_eq!(lexer.take()?, Tokens::IDENT("a".to_string()));

        assert_eq!(lexer.take()?, Tokens::ASSIGN);

        assert_eq!(lexer.take()?, Tokens::IDENT("b".to_string()));

        assert_eq!(lexer.take()?, Tokens::EOF);

        Ok(())
    }

    #[test]
    fn scans_keywords() -> Result<(), LexerError> {
        let input = "fn let if else for goto struct union u0 i8 i16 i32";

        let mut lexer = Lexer::new(input.as_bytes())?;

        assert_eq!(lexer.take()?, Tokens::FN);

        assert_eq!(lexer.take()?, Tokens::LET);

        assert_eq!(lexer.take()?, Tokens::IF);

        assert_eq!(lexer.take()?, Tokens::ELSE);

        assert_eq!(lexer.take()?, Tokens::FOR);

        assert_eq!(lexer.take()?, Tokens::GOTO);

        assert_eq!(lexer.take()?, Tokens::STRUCT);

        assert_eq!(lexer.take()?, Tokens::UNION);

        assert_eq!(lexer.take()?, Tokens::U0);

        assert_eq!(lexer.take()?, Tokens::I8);

        assert_eq!(lexer.take()?, Tokens::I16);

        assert_eq!(lexer.take()?, Tokens::I32);

        assert_eq!(lexer.take()?, Tokens::EOF);

        Ok(())
    }

    #[test]
    fn scans_semicolon() -> Result<(), LexerError> {
        let input = ";";

        let mut lexer = Lexer::new(input.as_bytes())?;

        assert_eq!(lexer.take()?, Tokens::SEMICOLON);

        assert_eq!(lexer.take()?, Tokens::EOF);

        Ok(())
    }

    #[test]
    fn scans_string() -> Result<(), LexerError> {
        let input = "\"string\"";

        let mut lexer = Lexer::new(input.as_bytes())?;

        assert_eq!(lexer.take()?, Tokens::STRING("string".to_string()));

        assert_eq!(lexer.take()?, Tokens::EOF);

        Ok(())
    }

    #[test]
    fn scans_char() -> Result<(), LexerError> {
        let input = "'c'";

        let mut lexer = Lexer::new(input.as_bytes())?;

        assert_eq!(lexer.take()?, Tokens::CHAR('c'));

        assert_eq!(lexer.take()?, Tokens::EOF);

        Ok(())
    }

    #[test]
    fn error_on_unterminated_string() -> Result<(), LexerError> {
        let input = "\"string";

        let mut lexer = Lexer::new(input.as_bytes())?;

        assert_eq!(lexer.take(), Result::Err(LexerError::UnterminatedString));

        Ok(())
    }

    #[test]
    fn error_on_unterminated_char() -> Result<(), LexerError> {
        let input = "'c";

        let mut lexer = Lexer::new(input.as_bytes())?;

        assert_eq!(lexer.take(), Result::Err(LexerError::UnterminatedCharacterConstant));

        Ok(())
    }

    #[test]
    fn error_on_unterminated_char2() -> Result<(), LexerError> {
        let input = "'";

        let mut lexer = Lexer::new(input.as_bytes())?;

        assert_eq!(lexer.take(), Result::Err(LexerError::UnterminatedCharacterConstant));

        Ok(())
    }

    #[test]
    fn error_on_long_char() -> Result<(), LexerError> {
        let input = "'cheese balls'";

        let mut lexer = Lexer::new(input.as_bytes())?;

        assert_eq!(lexer.take(), Result::Err(LexerError::CharacterConstantTooLong));

        Ok(())
    }

    #[test]
    fn error_on_integer_overflow() -> Result<(), LexerError> {
        let input = "2147483648";

        let mut lexer = Lexer::new(input.as_bytes())?;

        assert_eq!(lexer.take(), Result::Err(LexerError::IntegerOverflow));

        Ok(())
    }


}
