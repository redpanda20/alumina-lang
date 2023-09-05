use std::{fs, process::Command};


pub(crate) mod token;
pub(crate) mod parser;
pub(crate) mod generation;

use token::Lexer;
use parser::Parser;
use generation::Generator;

fn main() {
    let file = fs::File::open("test.alo")
        .expect("Unable to open file");

    let tokens = Lexer::parse(file)
        .expect("Error lexing file");
    println!("Tokens {:?}", tokens);

    let nodes = Parser::parse(tokens.into_iter())
        .expect("Error while parsing tokens");
    println!("Nodes {:?}", nodes);

    /* Code generator */
    let code = Generator::generate_program(nodes.into_iter())
        .expect("Error while generating code");
    println!("Generated code:\n{}", code);

    // create_executable(code);
}

fn create_executable(code: String) {
    
    /* Write to file */
    fs::create_dir_all("build").unwrap();
    fs::write("build/output.asm", code)
        .expect("Unable to write to file");

    /*
        Assembler
        nasm -felf64 output.asm
    */
    Command::new("nasm")
        .arg("-felf64")
        .arg("build/output.asm")
        .spawn()
        .unwrap()
        .wait()
        .expect("NASM assembler failed");

    /*
        Linker
        ld output.o -o output
    */
    Command::new("ld")
        .arg("-o")
        .arg("build/output")
        .arg("build/output.o")
        .spawn()
        .unwrap()
        .wait()
        .expect("GNU linker failed");
}