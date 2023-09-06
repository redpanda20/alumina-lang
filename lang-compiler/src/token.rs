use std::io;
use std::iter::Peekable;

use char_reader::CharReader;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Sep,
    Exit,
    Let,
    Ident(String),
    IntLiteral(u32),
    Eq,
    Plus,
    Minus,
    Star,
    FSlash,
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

pub struct Lexer<R: io::Read> {
    input: Peekable<CharReader<R>>,
    tokens: Vec<Token>
}

impl <R: io::Read>Lexer<R> {
    pub fn tokenize(reader: R) -> Result<Vec<Token>, LexerError> {

        let input = CharReader::new(reader)
            .into_iter().peekable();

        let mut lexer = Lexer {
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
            Some('+') => {
                self.tokens.push(Token::Plus);
                self.input.next();
            },
             Some('-') => {
                self.tokens.push(Token::Minus);
                self.input.next();
            },
            Some('*') => {
                self.tokens.push(Token::Star);
                self.input.next();
            },
            Some('/') => {
                self.tokens.push(Token::FSlash);
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
            Some(';') | Some('\n') => {
                self.tokens.push(Token::Sep);
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
            "exit" => self.tokens.push(Token::Exit),
            "let" => self.tokens.push(Token::Let),
            _ => self.tokens.push(Token::Ident(literal))
        };

        Ok(())
    }

}