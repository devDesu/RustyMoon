use std::fs;
use std::env;
mod core;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let result = fs::read(&args[0]).expect("Failed to read file");
    core::parser::parse_all(&result);
}
