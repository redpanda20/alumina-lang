use std::fs;


fn main() {
    let contents = fs::read_to_string("src/test.alo")
        .expect("Unable to open file");
    

    println!("Hello, world! {:?}", contents);
}
