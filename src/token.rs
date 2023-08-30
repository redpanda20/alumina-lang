use std::num::ParseIntError;


#[derive(Debug, Clone)]
pub enum Token {
    Return,
    Let,
    IntLiteral(u32),
    Eq
}

#[derive(Debug, Clone)]
pub enum TokenizerError {
    IntParse(std::num::ParseIntError)
}
impl From<ParseIntError> for TokenizerError {
    fn from(err: ParseIntError) -> TokenizerError {
        TokenizerError::IntParse(err)
    }
}


pub fn tokenize(input: &mut impl Iterator<Item = char>) -> Result<Vec<Token>, TokenizerError> {
    let mut tokens = Vec::new();

    while let Some(char) = input.next() {
        match char {
            '=' => tokens.push(Token::Eq),
            t if t.is_numeric() => {
                let mut num = t.to_string();
                input.take_while(|c| c.is_numeric())
                    .for_each(|ch| num.push(ch));
                tokens.push(Token::IntLiteral(num.parse::<u32>()?))
            },
            t if t.is_alphabetic() => {
                let mut literal = t.to_string();
                input.take_while(|c| c.is_alphanumeric())
                    .for_each(|ch| literal.push(ch));
                match literal.as_str() {
                    "return" => tokens.push(Token::Return),
                    "let" => tokens.push(Token::Let),
                    _ => { println!("{}", literal) }
                }
            }
            _ => {} // Whitespace and other weird characters
        }
    }
    return Ok(tokens);
}
