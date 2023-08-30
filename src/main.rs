use std::fs;

mod token;

fn main() {
    let data = fs::read_to_string("test.alo")
        .expect("Unable to read file");
    
    let tokens = token::tokenize(&mut data.chars()).unwrap();

    println!("Finished tokenising {:?}", tokens);
}
