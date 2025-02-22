use std::io;
use std::iter::Peekable;
use std::sync::Arc;

use char_reader::CharReader;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Sep,
    Exit,
    Let,
    If,
    Else,
    While,
    Ident(Arc<str>),
    IntLiteral(u32),
    Not,
    NotEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Plus,
    Minus,
    Star,
    FSlash,
    LParen,
    RParen,
    LBrace,
    RBrace
}

#[derive(Debug)]
pub enum LexerError {
    IntParse(std::num::ParseIntError),
    IOError(io::Error),
    UnexpectedCharacter(char),
    EndOfInput
}
impl From<std::num::ParseIntError> for LexerError {
    fn from(err: std::num::ParseIntError) -> LexerError {
        LexerError::IntParse(err)
    }
}
impl From<io::Error> for LexerError {
    fn from(err: io::Error) -> Self {
        LexerError::IOError(err)
    }
}

pub struct Lexer<R: io::Read> {
    input: Peekable<CharReader<R>>,
}

impl <R: io::Read>Lexer<R> {
    
    pub fn new(reader: R) -> Lexer<R> {
        let input = CharReader::new(reader).peekable();
        Lexer { input }
    }

    pub fn tokenize(reader: R) -> Result<Vec<Token>, LexerError> {

        let mut lexer = Lexer::new(reader);
        let mut tokens = Vec::new();

        loop {
            match lexer.parse_token() {
                Ok(token) => tokens.push(token),
                Err(LexerError::EndOfInput) => break,
                Err(err) => return Err(err)
            }
        }

        Ok(tokens)
    }


    fn parse_token(&mut self) -> Result<Token, LexerError> {
        let token = match self.input.next() {
            Some('!') => match self.input.next_if_eq(&'=') {
                None => Token::Not,
                Some(_) => Token::NotEqual
            }
            Some('=') => match self.input.next_if_eq(&'=') {
                None => Token::Equal,
                Some(_) => Token::EqualEqual
            },
            Some('>') => match self.input.next_if_eq(&'=') {
                None => Token::Greater,
                Some(_) => Token::GreaterEqual
            },
            Some('<') => match self.input.next_if_eq(&'=') {
                None => Token::Less,
                Some(_) => Token::LessEqual
            },
            Some('+') => Token::Plus,
            Some('-') => Token::Minus,
            Some('*') => Token::Star,
            Some('/') => Token::FSlash,
            Some('(') => Token::LParen,
            Some(')') => Token::RParen,
            Some('{') => Token::LBrace,
            Some('}') => Token::RBrace,
            Some(';') | Some('\n') => Token::Sep,
            Some(ch) if ch.is_numeric() => self.parse_int(ch)?,
            Some(ch) if ch.is_alphabetic() => self.parse_literal(ch)?,
            Some(ch) if ch.is_whitespace() => self.parse_whitespace()?,
            Some(ch) => return Err(LexerError::UnexpectedCharacter(ch)),
            None => return Err(LexerError::EndOfInput)
        };
        Ok(token)
    }

    fn parse_whitespace(&mut self) -> Result<Token, LexerError> {
        loop {
            match self.input.next_if(|ch| ch.is_whitespace()) {
                None => break,
                _ => ()
            }
        }
        self.parse_token()
    }

    fn parse_int(&mut self, first_char: char) -> Result<Token, LexerError> {
        let mut num = first_char.to_string();
        loop {
            match self.input.peek() {
                Some(ch) if ch.is_numeric() => {
                    num.push(*ch);
                    self.input.next();
                },
                Some(_) | None => break
            }
        }

        Ok(Token::IntLiteral(num.parse::<u32>()?))
    }

    fn parse_literal(&mut self, first_char: char) -> Result<Token, LexerError> {
        let mut literal = first_char.to_string();
        loop {
            match self.input.peek() {
                Some(ch) if ch.is_alphanumeric() => {
                    literal.push(*ch);
                    self.input.next();
                },
                Some(_) | None => break
            }
        }

        Ok(match literal.to_lowercase().as_str() {
            "exit" => Token::Exit,
            "let" => Token::Let,
            "if" => Token::If,
            "else" => Token::Else,
            "while" => Token::While,
            _ => Token::Ident(literal.into()),
        })
    }

}
impl<R: std::io::Read> Iterator for Lexer<R> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        self.parse_token().ok()
    }
}