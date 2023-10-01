//#![feature(rustc_attrs)]


use std::fs;
use std::env;
mod core;
mod structures;
mod vm;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let result = fs::read(&args[0]).expect("Failed to read file");
    let fn_info = core::parser::parse_all(&result);
    println!("{}", fn_info);
}
