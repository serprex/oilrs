extern crate fnv;
use std::collections::hash_map::Entry;
use std::env;
use std::fs;
use std::fmt::{self, Display, Write};
use std::i64;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use fnv::FnvHashMap;

#[derive(Clone, Eq, PartialEq)]
enum Value {
	S(Rc<String>),
	I(i64),
}

impl Display for Value {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Value::S(ref x) => write!(f, "{}", x),
			Value::I(x) => write!(f, "{}", x),
		}
	}
}

fn is_num(s: &str) -> bool {
	s == "0" || {
		let mut chs = s.bytes();
		match chs.next() {
			Some(b'-') => match chs.next() {
				Some(b'1'...b'9') => chs.all(|c| c >= b'0' && c <= b'9'),
				_ => false,
			},
			Some(b'1'...b'9') => chs.all(|c| c >= b'0' && c <= b'9'),
			_ => false,
		}
	}
}

fn num_decr(s: &mut Vec<u8>) {
	if let Some(&b'-') = s.first() {
		num_incr_core(s, 1)
	} else {
		num_decr_core(s, 0)
	}
}
fn num_incr(s: &mut Vec<u8>) {
	if let Some(&b'-') = s.first() {
		num_decr_core(s, 1)
	} else {
		num_incr_core(s, 0)
	}
}

fn num_decr_core(s: &mut Vec<u8>, start: usize) {
	for n in s[start..].iter_mut().rev() {
		if *n == b'0' {
			*n = b'9';
		} else {
			*n -= 1;
			break
		}
	}
	if s[start] == b'0' {
		s.remove(start);
	}
}

fn num_incr_core(s: &mut Vec<u8>, start: usize) {
	for n in s[start..].iter_mut().rev() {
		if *n == b'9' {
			*n = b'0';
		} else {
			*n += 1;
			return
		}
	}
	s.insert(start, b'1');
}

fn num_parse(s: &str) -> Option<i64> {
	if is_num(s) {
		s.parse::<i64>().ok()
	} else {
		None
	}
}

impl Value {
	pub fn incr(&mut self) {
		let newx;
		loop {
			match *self {
				Value::I(i64::MAX) => *self = Value::S(Rc::new(String::from("9223372036854775808"))),
				Value::I(x) => *self = Value::I(x+1),
				Value::S(ref mut x) => {
					if is_num(&x[..]) {
						let s = Rc::make_mut(x);
						unsafe { num_incr(s.as_mut_vec()); }
						if let Some(x) = num_parse(s) {
							newx = x;
							break
						}
					} else {
						newx = 1;
						break
					}
				},
			}
			return
		}
		*self = Value::I(newx);
	}

	pub fn decr(&mut self) {
		let newx;
		loop {
			match *self {
				Value::I(i64::MIN) => *self = Value::S(Rc::new(String::from("-9223372036854775809"))),
				Value::I(x) => *self = Value::I(x-1),
				Value::S(ref mut x) => {
					if is_num(&x[..]) {
						let s = Rc::make_mut(x);
						unsafe { num_decr(s.as_mut_vec()); }
						if let Some(x) = num_parse(s) {
							newx = x;
							break
						}
					} else {
						newx = -1;
						break
					}
				},
			}
			return
		}
		*self = Value::I(newx);
	}
}

impl<'a> From<&'a str> for Value {
	fn from(s: &'a str) -> Value {
		if let Some(x) = num_parse(&s) { Value::I(x) } else { Value::S(Rc::new(String::from(s))) }
	}
}

impl From<String> for Value {
	fn from(s: String) -> Value {
		if let Some(x) = num_parse(&s) { Value::I(x) } else { Value::S(Rc::new(s)) }
	}
}

impl From<char> for Value {
	fn from(c: char) -> Value {
		match c {
			'0'...'9' => Value::I((c as u32 - '0' as u32) as i64),
			_ => Value::S(Rc::new(c.to_string())),
		}
	}
}


struct Tape<'a> {
	pub idx: i64,
	pub tape: FnvHashMap<i64, Value>,
	pub dir: bool,
	pub root: Option<&'a Path>,
}

struct TapeChild<'a, 'b: 'a> {
	pub tape: Tape<'a>,
	pub parent: &'a mut Tape<'b>,
	pub iidx: i64,
	pub oidx: i64,
}

impl<'a> Tape<'a> {
	pub fn new() -> Tape<'a> {
		Tape {
			idx: 0,
			dir: true,
			tape: FnvHashMap::default(),
			root: None,
		}
	}
	pub fn step(&mut self) {
		self.idx += if self.dir { 1 } else { -1 };
	}
	pub fn read_val(&self, i: i64) -> Value {
		if let Some(x) = self.tape.get(&i) {
			x.clone()
		} else {
			Value::I(0)
		}
	}
	pub fn read_string(&self, i: i64) -> Rc<String> {
		match self.read_val(i) {
			Value::S(ref x) => x.clone(),
			Value::I(x) => Rc::new(x.to_string()),
		}
	}
	pub fn read_i64(&self) -> i64 {
		match self.tape.get(&self.idx) {
			Some(&Value::I(x)) => x,
			_ => 0,
		}
	}
	pub fn op1(&mut self) {
		self.step();
		let a = self.read_i64();
		let a = self.read_val(a);
		self.step();
		let b = self.read_i64();
		self.tape.insert(b, a);
	}
	pub fn op7(&mut self) {
		self.step();
		let a = self.read_i64();
		self.idx = if self.dir { self.idx + a } else { self.idx - a };
	}
	pub fn op8(&mut self) {
		self.step();
		let a = self.read_i64();
		match self.tape.entry(a) {
			Entry::Occupied(mut ent) => ent.get_mut().incr(),
			Entry::Vacant(ent) => { ent.insert(Value::I(1)); },
		}
	}
	pub fn op9(&mut self) {
		self.step();
		let a = self.read_i64();
		match self.tape.entry(a) {
			Entry::Occupied(mut ent) => ent.get_mut().decr(),
			Entry::Vacant(ent) => { ent.insert(Value::I(-1)); },
		}
	}
	pub fn op10(&mut self) {
		self.step();
		let ai = self.read_i64();
		let a = self.read_val(ai);
		self.step();
		let bi = self.read_i64();
		let b = self.read_val(bi);
		if a != b {
			self.step();
		}
		self.step();
		self.idx = self.read_i64();
	}
	pub fn op12(&mut self) {
		self.step();
		let a = self.read_i64();
		let a = self.read_string(a);
		self.step();
		let mut b = self.read_i64();
		self.tape.insert(b, Value::I(a.len() as i64));
		for ch in a.chars() {
			b += 1;
			self.tape.insert(b, Value::from(ch));
		}
	}
	pub fn op13(&mut self) {
		self.step();
		let a = self.read_i64();
		self.step();
		let b = self.read_i64();
		self.step();
		let c = self.read_i64();
		let mut s = String::new();
		if b > 0 {
			for b in 0..b {
				write!(s, "{}", self.read_val(a+b)).ok();
			}
		}
		self.tape.insert(c, Value::from(s));
	}
	fn run_path(&mut self, oi: i64, ii: i64, path: &Path, modcache: &mut FnvHashMap<PathBuf, FnvHashMap<i64, Value>>) {
		let mut child = TapeChild {
			tape: Tape::new(),
			parent: self,
			oidx: oi,
			iidx: ii,
		};
		child.tape.root = path.parent();
		if let Some(m) = modcache.get_mut(path).map(|m| m.clone()) {
			child.tape.tape = m;
		}
		else if let Ok(f) = fs::File::open(&path) {
			let f = BufReader::new(f);
			for (idx, line) in f.lines().enumerate() {
				if let Ok(line) = line {
					child.tape.tape.insert(idx as i64, Value::from(line));
				}
			}
			modcache.insert(path.to_path_buf(), child.tape.tape.clone());
		}
		child.run(modcache);
	}
	pub fn op14(&mut self, modcache: &mut FnvHashMap<PathBuf, FnvHashMap<i64, Value>>) {
		self.step();
		let a = self.read_string(self.idx);
		self.step();
		let oi = self.read_i64();
		self.step();
		let ii = self.read_i64();
		if let Some(pref) = self.root {
			self.run_path(oi, ii, pref.join(&a[..]).as_path(), modcache);
		} else {
			self.run_path(oi, ii, Path::new(&a[..]), modcache);
		}
	}
	pub fn run(&mut self)
	{
		let mut modcache = FnvHashMap::default();
		loop {
			match self.tape.get(&self.idx) {
				Some(&Value::I(cell)) => {
					match cell {
						1 => self.op1(),
						2 => self.dir ^= true,
						3 => return,
						4 => {
							self.step();
							let a = self.read_i64();
							print!("{}", self.read_val(a));
						}
						5 => {
							(&mut io::stdout() as &mut io::Write).flush().ok();
							let stdin = io::stdin();
							let mut inlock = stdin.lock();
							let mut s = String::new();
							if inlock.read_line(&mut s).is_ok() {
								if s.ends_with('\n') {
									let len = s.len() - 1;
									s.truncate(len);
								}
								self.step();
								let a = self.read_i64();
								self.tape.insert(a, Value::from(s));
							}
						},
						6 => {
							self.step();
							self.idx = self.read_i64();
							continue
						},
						7 => {
							self.op7();
							continue
						},
						8 => self.op8(),
						9 => self.op9(),
						10 => {
							self.op10();
							continue
						},
						11 => println!(""),
						12 => self.op12(),
						13 => self.op13(),
						14 => self.op14(&mut modcache),
						_ => (),
					}
				},
				Some(_) => (),
				_ => return,
			}
			self.step();
		}
	}
}

impl<'p, 'pl> TapeChild<'p, 'pl> {
	pub fn step(&mut self) {
		self.tape.step()
	}
	pub fn read_val(&self, i: i64) -> Value {
		self.tape.read_val(i)
	}
	pub fn read_i64(&self) -> i64 {
		self.tape.read_i64()
	}
	pub fn run(&mut self, modcache: &mut FnvHashMap<PathBuf, FnvHashMap<i64, Value>>)
	{
		loop {
			match self.tape.tape.get(&self.tape.idx) {
				Some(&Value::I(cell)) => {
					match cell {
						1 => self.tape.op1(),
						2 => self.tape.dir ^= true,
						3 => return,
						4 => {
							self.step();
							let a = self.read_i64();
							let a = self.read_val(a);
							self.parent.tape.insert(self.oidx, a);
							self.oidx += if self.parent.dir { 1 } else { -1 };
						}
						5 => {
							self.step();
							let a = self.read_i64();
							let v = if let Some(x) = self.parent.tape.get(&self.iidx) {
								x.clone()
							} else {
								Value::I(0)
							};
							self.tape.tape.insert(a, v);
							self.iidx += if self.parent.dir { 1 } else { -1 };
						},
						6 => {
							self.step();
							self.tape.idx = self.read_i64();
							continue
						},
						7 => {
							self.tape.op7();
							continue
						},
						8 => self.tape.op8(),
						9 => self.tape.op9(),
						10 => {
							self.tape.op10();
							continue
						},
						12 => self.tape.op12(),
						13 => self.tape.op13(),
						14 => self.tape.op14(modcache),
						_ => (),
					}
				},
				Some(_) => (),
				_ => return,
			}
			self.step();
		}
	}
}

fn main() {
	if let Some(a) = env::args().nth(1) {
		let mut tape = Tape::new();
		let path = Path::new(&a);
		if let Ok(f) = fs::File::open(&path) {
			let f = BufReader::new(f);
			for (idx, line) in f.lines().enumerate() {
				if let Ok(line) = line {
					tape.tape.insert(idx as i64, Value::from(line));
				}
			}
		}
		tape.root = path.parent();
		tape.run();
	} else {
		println!("oilrs [filename]");
	}
}
