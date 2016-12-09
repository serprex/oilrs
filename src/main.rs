extern crate fnv;
mod tape;
mod value;
use std::env;
use std::fs;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use tape::Tape;
use value::Value;

fn main() {
	if let Some(a) = env::args().nth(1) {
		let mut tape = Tape::new();
		let path = Path::new(&a);
		if let Ok(f) = fs::File::open(&path) {
			let f = BufReader::new(f);
			for (idx, line) in f.lines().enumerate() {
				if let Ok(line) = line {
					tape.tape.insert(Value::I(idx as i64), Value::from(line));
				}
			}
		}
		tape.root = path.parent();
		tape.run();
	} else {
		println!("oilrs [filename]");
	}
}
