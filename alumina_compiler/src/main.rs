use std::{fs, process, env};

extern crate char_reader;

pub(crate) mod token;
pub(crate) mod parser;
pub(crate) mod generation;

use token::Lexer;
use parser::Parser;
use generation::Generator;

#[derive(Debug)]
enum CLIError {
    IO(std::io::Error),
    Lexer(token::LexerError),
    Parser(parser::ParserError),
    CodeGenerator(generation::GeneratorError)
}
impl From<std::io::Error> for CLIError {
    fn from(value: std::io::Error) -> Self {
        CLIError::IO(value)
    }
}
impl From<token::LexerError> for CLIError {
    fn from(value: token::LexerError) -> Self {
        CLIError::Lexer(value)
    }
}
impl From<parser::ParserError> for CLIError {
    fn from(value: parser::ParserError) -> Self {
        CLIError::Parser(value)
    }
}
impl From<generation::GeneratorError> for CLIError {
    fn from(value: generation::GeneratorError) -> Self { CLIError::CodeGenerator(value) }
}


fn main() -> Result<(), CLIError> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        println!("Alumina compiler");
        println!();
        println!("Usage: alumina-compiler [FILE] [options]?");
        println!("Options:");
        println!("  -tokens");
        println!("  -parse-tree");
        return Ok(())
    }
    
    println!(" \x1b[1;32m Compiling \x1b[0m '{}'...", &args[1]);
    let file = fs::File::open(&args[1])?;

    print!("   \x1b[1;34m Parsing \x1b[0m tokens...\r");
    let lexer;
    if args.iter().any(|i| i == "-tokens") {
        println!("");
        for token in Lexer::tokenize(file)? {
            println!("{token:?}")
        }
        return Ok(());
    } else {
        lexer = Lexer::new(file);    
    }

    print!("  \x1b[1;34m Building \x1b[0m parse tree...\r");
    let nodes = Parser::parse(lexer)?;
    if args.iter().any(|i| i == "-parse-tree") {
        println!("");
        for (i, node) in nodes.iter().enumerate() {
            println!("{i:<6} {node}");
        }
        return Ok(())
    }

    print!("\x1b[1;34m Generating \x1b[0m intermediate code...\r");
    let code = Generator::generate_program(nodes.into_iter())?;

    fs::create_dir_all("build")?;
    fs::write("build/output.asm", code)?;

    print!("  \x1b[1;34m Building \x1b[0m binary...\r");
    /* Assembler
        nasm -f <elf64 | win64> output.asm
    */
    process::Command::new("nasm")
        .arg(
            if cfg!(target_family = "windows") {r"-fwin64"} else {r"-felf64"}
        )
        .arg("build/output.asm")
        .spawn()?
        .wait()?;

    /* Linker
    Linux: GNU Linker (ld)
    */
    #[cfg(target_family = "unix")]
    process::Command::new("ld")
        .arg("-o")
        .arg("build/output")
        .arg("build/output.o")
        .spawn()?
        .wait()?;

    println!("  \x1b[1;32m Finished \x1b[0m compiling '{}' successfully", &args[1]);
    Ok(())
}