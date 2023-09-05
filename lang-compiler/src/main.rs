use std::{fs, process::Command};

mod token;
use token::Lexer;

mod parser;
use parser::Parser;

fn codegen() -> String {
    let mut output = String::new();
    output += "global _start\n";
    output += "_start:\n";
	output += "mov rdi, 69\n";
	output += "mov rax, 60\n";
	output += "syscall\n";
    return output;
}

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
    // let code = codegen();
    
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