use std::io::{Read, BufReader};
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone)]
pub enum Token {
    Return,
    Let,
    Literal(String),
    IntLiteral(u32),
    Eq,
    LParen,
    RParen
}

#[derive(Debug)]
pub enum LexerError {
    IntParse(std::num::ParseIntError),
    IOError(std::io::Error),
    UnexpectedCharacter(char),
    EndOfInput
}
impl From<std::num::ParseIntError> for LexerError {
    fn from(err: std::num::ParseIntError) -> LexerError {
        LexerError::IntParse(err)
    }
}
impl From<std::io::Error> for LexerError {
    fn from(err: std::io::Error) -> Self {
        LexerError::IOError(err)
    }
}

pub struct TokenLexer<'a> {
    input: Peekable<Chars<'a>>,
    tokens: Vec<Token>
}

impl TokenLexer<'_> {
    pub fn parse<R: Read>(mut reader: BufReader<R>) -> Result<Vec<Token>, LexerError> {

        let mut string = String::new();
        reader.read_to_string(&mut string)?;

        let input = string.chars().peekable();

        let mut lexer = TokenLexer {
            input,
            tokens: Vec::new()
        };

        loop {
            match lexer.parse_token() {
                Ok(_) => (),
                Err(LexerError::EndOfInput) => break,
                Err(err) => return Err(err)
            }
        }

        Ok(lexer.tokens)
    }


    fn parse_token(&mut self) -> Result<(), LexerError> {
        match self.input.peek() {
            Some('=') => {
                self.tokens.push(Token::Eq);
                self.input.next();
            },
            Some('(') => {
                self.tokens.push(Token::LParen);
                self.input.next();
            },
            Some(')') => {
                self.tokens.push(Token::RParen);
                self.input.next();
            },
            Some(ch) if ch.is_numeric() => {
                self.parse_int()?;
            },
            Some(ch) if ch.is_alphabetic() => {
                self.parse_literal()?;
            },
            Some(ch) if ch.is_whitespace() => {
                self.input.next();
            }
            Some(ch) => return Err(LexerError::UnexpectedCharacter(*ch)),
            None => return Err(LexerError::EndOfInput)
        }

        Ok(())
    }

    fn parse_int(&mut self) -> Result<(), LexerError> {
        let mut num = String::new();
        loop {
            match self.input.peek() {
                Some(ch) if ch.is_numeric() => {
                    num.push(*ch);
                    self.input.next();
                },
                Some(_) | None => break
            }
        }

        self.tokens.push(Token::IntLiteral(num.parse::<u32>()?));

        Ok(())
    }

    fn parse_literal(&mut self) -> Result<(), LexerError> {
        let mut literal = String::new();
        loop {
            match self.input.peek() {
                Some(ch) if ch.is_alphanumeric() => {
                    literal.push(*ch);
                    self.input.next();
                },
                Some(_) | None => break
            }
        }

        match literal.as_str() {
            "return" => self.tokens.push(Token::Return),
            "let" => self.tokens.push(Token::Let),
            _ => self.tokens.push(Token::Literal(literal))
        };

        Ok(())
    }

}