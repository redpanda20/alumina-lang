use std::{fs, process, env};
use process::Command;

extern crate char_reader;

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
            Self::NotEnoughArguments => write!(f, "usage: alumina-compiler [file]"),
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

    println!("\x1b[1;32m Compiling \x1b[0m '{}'...", &args[1]);
    let file = fs::File::open(&args[1])?;

    print!("Parsing tokens...\r");
    let tokens = Lexer::tokenize(file)?;

    print!("Building parse tree...\r");
    let nodes = Parser::parse(tokens.into_iter())?;

    print!("Generating intermediate code...\r");
    let code = Generator::generate_program(nodes.into_iter())?;

    fs::create_dir_all("build")?;
    fs::write("build/output.asm", code)?;

    print!("Building binary...\r");
    /* Assembler
        nasm -f <elf64 | win64> output.asm
    */
    Command::new("nasm")
        .arg(
            if cfg!(target_family = "windows") {"-fwin64"} else {"-felf64"}
        )
        .arg("build/output.asm")
        .spawn()?
        .wait()?;

    /* Linker
    Linux: GNU Linker (ld)
    Win  : Visual Studio Linker
    */
    #[cfg(not(any(target_family = "unix", target_family = "windows")))]
    panic!("Platform not supported");
    
    #[cfg(target_family = "unix")]
    Command::new("ld")
        .arg("-o")
        .arg("build/output")
        .arg("build/output.o")
        .spawn()?
        .wait()?;   
    #[cfg(target_family = "windows")]
    Command::new(r"C:\Program Files (x86)\Microsoft Visual Studio\2019\Community\VC\Tools\MSVC\14.29.30133\bin\Hostx64\x64\link.exe")
        .args(["/subsystem:console", "/nodefaultlib", "/entry:_start", "/manifest", "/nologo"])
        .arg(r".\build\output.obj")
        .arg(r"/OUT:.\build\output.exe")
        .spawn()?
        .wait()?;    
    

    println!(" \x1b[1;32m Finished \x1b[0m compiling '{}' successfully", &args[1]);
    Ok(())
}