# Alumina Language

![Image of a piece of Corundum mineral](https://upload.wikimedia.org/wikipedia/commons/1/1e/Corindon_azulEZ.jpg)

## Description
Alumina is a hobbyist programming language. Alumina is named after the most common natural oxide of Aluminium (Al<sub>2</sub>O<sub>3</sub>)[^al2o3]. Much like its elemental inspiration Alumina is lightweight, abrasive, and refuses to catch fire.

This project was inspired by Hydrogen language project[^hydro] and built in [Rust](https://www.rust-lang.org/).

## Building
Requires `cargo`, `nasm`, and `ld` on a Linux operating system. Use `cargo build --release` from the project root to build the compiler. To run the compiler use either:
- `alumina-compiler [file]` 
- `cargo run --release -- [file]` 

Build artifacts can be found in the `/build` directory


## Contributing
I am not accepting pull requests. This may change as the project continues.


## Project status
- [x] Ongoing


## See more

[^al2o3]: Alumina [(Al~2~O~3~)](https://en.wikipedia.org/wiki/Aluminium_oxide)

[^hydro]: [Orosmatthew's language](https://github.com/orosmatthew/hydrogen-cpp/tree/master), I would highly recommend checking his series out.