extern crate fnv;
mod tape;
mod value;
use std::borrow::Cow;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use fnv::FnvHashMap;
use tape::Tape;
use value::Value;

fn main() {
	if let Some(a) = env::args().nth(1) {
		if let Some(b) = env::args().nth(2) {
			if let Ok(f) = fs::File::open(&a) {
				let f = BufReader::new(f);
				let mut lines = Vec::new();
				let mut labels = FnvHashMap::default();
				let mut labelfill = Vec::new();
				for line in f.lines() {
					if let Ok(line) = line {
						let lineno = lines.len();
						lines.push(if let Some(op) = match &line[..] {
							"nop" => Some("0"),
							"copy" | "mov" => Some("1"),
							"reverse" => Some("2"),
							"quit" | "exit" | "return" => Some("3"),
							"output" | "write" => Some("4"),
							"user_input" | "read" => Some("5"),
							"jump" | "jmp" => Some("6"),
							"relative_jump" | "jr"  => Some("7"),
							"increment" | "+" => Some("8"),
							"decrement" | "-" => Some("9"),
							"conditional_jump" | "je" => Some("10"),
							"newline" => Some("11"),
							"explode" => Some("12"),
							"implode" => Some("13"),
							"call" => Some("14"),
							_ => None,
						} {
							Cow::Borrowed(op)
						} else if line.starts_with('$') {
							labelfill.push(lineno);
							Cow::Owned(line)
						} else if line.starts_with(':') {
							if let Some(oldidx) = labels.insert(String::from(&line[1..]), lineno.to_string()) {
								println!("Duplicate labels: {} {}", oldidx, lineno);
							}
							continue
						} else if line.starts_with('"') {
							Cow::Owned(String::from(&line[1..]))
						} else if line.starts_with('#') {
							continue
						} else {
							Cow::Owned(line)
						});
					}
				}
				if let Ok(mut output) = fs::File::create(&b) {
					let mut labelidx = 0;
					for (idx, line) in lines.into_iter().enumerate() {
						if labelidx < labelfill.len() && labelfill[labelidx] == idx {
							labelidx += 1;
							if let Some(lineno) = labels.get(&line[1..]) {
								writeln!(output, "{}", lineno).ok();
								continue
							} else {
								println!("Unknown label: {}", &line[1..]);
							}
						}
						writeln!(output, "{}", line).ok();
					}
				}
			}
		} else {
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
		}
	} else {
		println!("oilrs [filename]");
	}
}
