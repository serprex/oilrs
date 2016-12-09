extern crate fnv;
mod value;
use std::collections::hash_map::Entry;
use std::env;
use std::fs;
use std::fmt::Write;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use fnv::FnvHashMap;
use value::{is_num, Value};

struct Tape<'a> {
	pub idx: Value,
	pub tape: FnvHashMap<Value, Value>,
	pub dir: bool,
	pub root: Option<&'a Path>,
}

struct TapeChild<'a, 'b: 'a> {
	pub tape: Tape<'a>,
	pub parent: &'a mut Tape<'b>,
	pub iidx: Value,
	pub oidx: Value,
}

impl<'a> Tape<'a> {
	pub fn new() -> Tape<'a> {
		Tape {
			idx: Value::I(0),
			dir: true,
			tape: FnvHashMap::default(),
			root: None,
		}
	}
	pub fn step(&mut self) {
		if self.dir {
			self.idx.incr()
		} else {
			self.idx.decr()
		}
	}
	pub fn read_val(&self, i: &Value) -> Value {
		if let Some(x) = self.tape.get(&i) {
			x.clone()
		} else {
			Value::I(0)
		}
	}
	pub fn read_string(&self, i: &Value) -> Rc<String> {
		match self.read_val(i) {
			Value::S(ref x) => x.clone(),
			Value::I(x) => Rc::new(x.to_string()),
		}
	}
	pub fn read_int(&self) -> Value {
		match self.tape.get(&self.idx) {
			Some(&Value::I(x)) => Value::I(x),
			Some(&Value::S(ref s)) if is_num(&s[..]) => Value::S(s.clone()),
			_ => Value::I(0),
		}
	}
	pub fn op1(&mut self) {
		self.step();
		let a = self.read_int();
		let a = self.read_val(&a);
		self.step();
		let b = self.read_int();
		self.tape.insert(b, a);
	}
	pub fn op7(&mut self) {
		self.step();
		let a = self.read_int();
		self.idx = if self.dir {
			self.idx.incr_by(&a)
		} else {
			self.idx.decr_by(&a)
		};
	}
	pub fn op8(&mut self) {
		self.step();
		let a = self.read_int();
		match self.tape.entry(a) {
			Entry::Occupied(mut ent) => ent.get_mut().incr(),
			Entry::Vacant(ent) => { ent.insert(Value::I(1)); },
		}
	}
	pub fn op9(&mut self) {
		self.step();
		let a = self.read_int();
		match self.tape.entry(a) {
			Entry::Occupied(mut ent) => ent.get_mut().decr(),
			Entry::Vacant(ent) => { ent.insert(Value::I(-1)); },
		}
	}
	pub fn op10(&mut self) {
		self.step();
		let a = self.read_int();
		let a = self.read_val(&a);
		self.step();
		let b = self.read_int();
		let b = self.read_val(&b);
		if a != b {
			self.step();
		}
		self.step();
		self.idx = self.read_int();
	}
	pub fn op12(&mut self) {
		self.step();
		let a = self.read_int();
		let a = self.read_string(&a);
		self.step();
		let mut b = self.read_int();
		self.tape.insert(b.clone(), Value::I(a.len() as i64));
		for ch in a.chars() {
			b.incr();
			self.tape.insert(b.clone(), Value::from(ch));
		}
	}
	pub fn op13(&mut self) {
		self.step();
		let mut a = self.read_int();
		self.step();
		let mut b = self.read_int();
		self.step();
		let c = self.read_int();
		let mut s = String::new();
		while b.gtz() {
			write!(s, "{}", self.read_val(&a)).ok();
			a.incr();
			b.decr();
		}
		self.tape.insert(c, Value::from(s));
	}
	fn run_path(&mut self, oi: Value, ii: Value, path: &Path, modcache: &mut FnvHashMap<PathBuf, FnvHashMap<Value, Value>>) {
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
					child.tape.tape.insert(Value::I(idx as i64), Value::from(line));
				}
			}
			modcache.insert(path.to_path_buf(), child.tape.tape.clone());
		}
		child.run(modcache);
	}
	pub fn op14(&mut self, modcache: &mut FnvHashMap<PathBuf, FnvHashMap<Value, Value>>) {
		self.step();
		let a = self.read_string(&self.idx);
		self.step();
		let oi = self.read_int();
		self.step();
		let ii = self.read_int();
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
							let a = self.read_int();
							print!("{}", self.read_val(&a));
						}
						5 => {
							(&mut io::stdout() as &mut io::Write).flush().ok();
							let stdin = io::stdin();
							let mut inlock = stdin.lock();
							let mut s = String::new();
							inlock.read_line(&mut s).ok();
							if s.ends_with('\n') {
								let len = s.len() - 1;
								s.truncate(len);
							}
							self.step();
							let a = self.read_int();
							self.tape.insert(a, Value::from(s));
						},
						6 => {
							self.step();
							self.idx = self.read_int();
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
	pub fn read_val(&self, i: &Value) -> Value {
		self.tape.read_val(i)
	}
	pub fn read_int(&self) -> Value {
		self.tape.read_int()
	}
	pub fn run(&mut self, modcache: &mut FnvHashMap<PathBuf, FnvHashMap<Value, Value>>)
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
							let a = self.read_int();
							let a = self.read_val(&a);
							self.parent.tape.insert(self.oidx.clone(), a);
							if self.parent.dir {
								self.oidx.incr();
							} else {
								self.oidx.decr();
							}
						}
						5 => {
							self.step();
							let a = self.read_int();
							let v = if let Some(x) = self.parent.tape.get(&self.iidx) {
								x.clone()
							} else {
								Value::I(0)
							};
							self.tape.tape.insert(a, v);
							if self.parent.dir {
								self.iidx.incr();
							} else {
								self.iidx.decr();
							}
						},
						6 => {
							self.step();
							self.tape.idx = self.read_int();
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
