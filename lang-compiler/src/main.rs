use std::{fs, process, env};
use process::Command;

pub(crate) mod token;
pub(crate) mod parser;
pub(crate) mod generation;

use token::Lexer;
use parser::Parser;
use generation::Generator;

enum CLIError {
    NotEnoughArguments,
    IOError(std::io::Error),
    LexerError(token::LexerError),
    ParserError(parser::ParserError),
    CodeGeneratorError(generation::GeneratorError)
}
impl std::fmt::Debug for CLIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotEnoughArguments => write!(f, "usage: lang-compiler [file]"),
            Self::IOError(arg0) => f.debug_tuple("IOError").field(arg0).finish(),
            Self::LexerError(arg0) => f.debug_tuple("LexerError").field(arg0).finish(),
            Self::ParserError(arg0) => f.debug_tuple("ParserError").field(arg0).finish(),
            Self::CodeGeneratorError(arg0) => f.debug_tuple("CodeGeneratorError").field(arg0).finish(),
        }
    }
}
impl From<std::io::Error> for CLIError {
    fn from(value: std::io::Error) -> Self {
        return CLIError::IOError(value)
    }
}
impl From<token::LexerError> for CLIError {
    fn from(value: token::LexerError) -> Self {
        return CLIError::LexerError(value)
    }
}
impl From<parser::ParserError> for CLIError {
    fn from(value: parser::ParserError) -> Self {
        return CLIError::ParserError(value)
    }
}
impl From<generation::GeneratorError> for CLIError {
    fn from(value: generation::GeneratorError) -> Self {
        return CLIError::CodeGeneratorError(value)
    }
}


fn main() -> Result<(), CLIError> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        return Err(CLIError::NotEnoughArguments)
    }

    println!("Compiling '{}'...", &args[1]);
    let file = fs::File::open(&args[1])?;

    println!("Parsing tokens...");
    let tokens = Lexer::tokenize(file)?;

    println!("Building parse tree...");
    let nodes = Parser::parse(tokens.into_iter())?;

    println!("Generating intermediate code...");
    let code = Generator::generate_program(nodes.into_iter())?;

    fs::create_dir_all("build")?;
    fs::write("build/output.asm", code)?;

    println!("Building binary...");
    /*
        Assembler
        nasm -felf64 output.asm
    */
    Command::new("nasm")
        .arg("-felf64")
        .arg("build/output.asm")
        .spawn()?
        .wait()?;

    /*
        Linker
        ld output.o -o output
    */
    Command::new("ld")
        .arg("-o")
        .arg("build/output")
        .arg("build/output.o")
        .spawn()?
        .wait()?;

    Ok(())
}