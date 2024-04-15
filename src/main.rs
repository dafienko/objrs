use std::env;
use objrs::run;

pub fn main() {
	let args: Vec<String> = env::args().collect();
	let def = String::from("hello");
    pollster::block_on(run(args.get(1).unwrap_or(&def))); 
}