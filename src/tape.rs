use std::collections::hash_map::Entry;
use std::char;
use std::fs;
use std::fmt::Write;
use std::i64;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use fnv::FnvHashMap;
use rand::{thread_rng, Rng};
use super::value::{is_num, Value};

pub struct Tape<'a> {
	pub idx: Value,
	pub tape: FnvHashMap<Value, Value>,
	pub dir: bool,
	pub root: &'a Path,
}

struct TapeChild<'a, 'b: 'a> {
	pub tape: Tape<'a>,
	pub parent: &'a mut Tape<'b>,
	pub iidx: Value,
	pub oidx: Value,
}

impl<'a> Tape<'a> {
	pub fn new(root: &'a Path) -> Tape<'a> {
		Tape {
			idx: Value::I(0),
			dir: true,
			tape: FnvHashMap::default(),
			root: root,
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
		match self.read_int() {
			Value::I(b) => {
				self.step();
				let c = self.read_int();
				let mut s = String::new();
				for _ in 0..b {
					write!(s, "{}", self.read_val(&a)).ok();
					a.incr();
				}
				self.tape.insert(c, Value::from(s));
			}
			_ => self.step(),
		}
	}
	pub fn op14(&mut self, modcache: &mut FnvHashMap<PathBuf, FnvHashMap<Value, Value>>) {
		self.step();
		let a = self.read_string(&self.idx);
		self.step();
		let oi = self.read_int();
		self.step();
		let ii = self.read_int();
		let path = self.root.join(&a[..]);
		let cachetape;
		{
			let mut child = TapeChild {
				tape: Tape::new(path.parent().unwrap_or_else(|| Path::new(""))),
				parent: self,
				oidx: oi,
				iidx: ii,
			};
			if let Some(m) = modcache.get_mut(&path).map(|m| m.clone()) {
				child.tape.tape = m;
				child.run(modcache);
				return
			}
			else if let Ok(f) = fs::File::open(&path) {
				let f = BufReader::new(f);
				for (idx, line) in f.lines().enumerate() {
					if let Ok(line) = line {
						child.tape.tape.insert(Value::I(idx as i64), Value::from(line));
					}
				}
				cachetape = child.tape.tape.clone();
				child.run(modcache);
			} else {
				return
			}
		}
		modcache.insert(path, cachetape);
	}
	pub fn op15(&mut self) {
		self.step();
		let a = self.read_int();
		let val = self.read_val(&a);
		self.tape.insert(a, match val {
			Value::I(i64::MAX) => Value::I(thread_rng().gen()),
			Value::I(x) if x > 0 => Value::I(thread_rng().gen_range(0, x+1)),
			_ => return,
		});
	}
	pub fn op16(&mut self) {
		self.step();
		let a = self.read_int();
		let a = self.read_string(&a);
		self.step();
		let mut b = self.read_int();
		self.tape.insert(b.clone(), Value::I(a.len() as i64));
		for ch in a.chars() {
			b.incr();
			self.tape.insert(b.clone(), Value::I(ch as u32 as i64));
		}
	}
	pub fn op17(&mut self) {
		self.step();
		let mut a = self.read_int();
		self.step();
		match self.read_int() {
			Value::I(b) => {
				self.step();
				let c = self.read_int();
				let mut s = String::with_capacity(b as usize);
				for _ in 0..b {
					s.push(match self.read_val(&a) {
						Value::I(x) if x >= 0 && x <= 0x10ffff => char::from_u32(x as u32).unwrap_or('\u{fffd}'),
						_ => '\u{fffd}'
					});
					a.incr();
				}
				self.tape.insert(c, Value::from(s));
			}
			_ => self.step(),
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
						15 => self.op15(),
						16 => self.op16(),
						17 => self.op17(),
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
						15 => self.tape.op15(),
						16 => self.tape.op16(),
						17 => self.tape.op17(),
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
