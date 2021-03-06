mod stdlib;
mod tape;
mod value;

use std::borrow::Cow;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use fxhash::FxHashMap;
use tape::Tape;
use value::Value;

fn main() {
	if let Some(a) = env::args().nth(1) {
		if let Some(b) = env::args().nth(2) {
			if let Ok(f) = fs::File::open(&a) {
				let f = BufReader::new(f);
				let mut lines = Vec::new();
				let mut labels = FxHashMap::default();
				let mut labelfill = Vec::new();
				for line in f.lines() {
					if let Ok(line) = line {
						let lineno = lines.len();
						lines.push(
							if let Some(op) = match &line[..] {
								"nop" => Some("0"),
								"copy" | "mov" => Some("1"),
								"reverse" => Some("2"),
								"quit" | "exit" | "return" => Some("3"),
								"output" | "write" => Some("4"),
								"user_input" | "read" => Some("5"),
								"jump" | "jmp" => Some("6"),
								"relative_jump" | "jr" => Some("7"),
								"increment" | "+" => Some("8"),
								"decrement" | "-" => Some("9"),
								"conditional_jump" | "je" => Some("10"),
								"newline" => Some("11"),
								"explode" => Some("12"),
								"implode" => Some("13"),
								"call" => Some("14"),
								"rand" => Some("15"),
								"ord" => Some("16"),
								"chr" => Some("17"),
								_ => None,
							} {
								Cow::Borrowed(op)
							} else if line.starts_with('$') {
								labelfill.push(lineno);
								Cow::Owned(line)
							} else if line.starts_with(':') {
								if let Some(oldidx) =
									labels.insert(String::from(&line[1..]), lineno.to_string())
								{
									println!("Duplicate labels: {} {}", oldidx, lineno);
								}
								continue;
							} else if line.starts_with('"') {
								Cow::Owned(String::from(&line[1..]))
							} else if line.starts_with('#') {
								continue;
							} else {
								Cow::Owned(line)
							},
						);
					}
				}
				if let Ok(mut output) = fs::File::create(&b) {
					let mut labelidx = 0;
					for (idx, line) in lines.into_iter().enumerate() {
						if labelidx < labelfill.len() && labelfill[labelidx] == idx {
							labelidx += 1;
							if let Some(lineno) = labels.get(&line[1..]) {
								writeln!(output, "{}", lineno).ok();
								continue;
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
			let mut tape = Tape::new(path.parent());
			if let Ok(f) = fs::File::open(&path) {
				let mut f = BufReader::new(f);
				let mut line = String::new();
				let mut idx = 0;
				while let Ok(n) = f.read_line(&mut line) {
					if n == 0 {
						break;
					}
					tape.tape
						.insert(Value::I(idx), Value::from(line.trim_end_matches('\n')));
					line.clear();
					idx += 1;
				}
			}
			tape.run();
		}
	} else {
		println!("oilrs [filename]: execute oil script");
		println!("oilrs [gas-file] [oil-output]: compile gas-file to oil-output");
	}
}
