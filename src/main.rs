use std::env;
use objrs::run;

pub fn main() {
	let args: Vec<String> = env::args().collect();
    pollster::block_on(run(args.get(1).unwrap()));
}