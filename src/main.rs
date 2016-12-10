extern crate fnv;
mod tape;
mod value;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use tape::Tape;
use value::Value;

fn main() {
	if let Some(a) = env::args().nth(1) {
		let path = Path::new(&a);
		let mut tape = Tape::new(path.parent().unwrap_or_else(|| Path::new("")));
		if let Ok(f) = fs::File::open(&path) {
			let f = BufReader::new(f);
			for (idx, line) in f.lines().enumerate() {
				if let Ok(line) = line {
					tape.tape.insert(Value::I(idx as i64), Value::from(line));
				}
			}
		}
		tape.run();
	} else {
		println!("oilrs [filename]");
	}
}
