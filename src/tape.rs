use super::stdlib::gen_libs;
use super::value::{is_num, num_gtz, Value, ValueAsChars};
use fxhash::FxHashMap;
use rand::distributions::{uniform, Distribution};
use rand::{thread_rng, Rng};
use std::char;
use std::cmp::{Ord, Ordering};
use std::collections::hash_map::Entry;
use std::fmt::Write;
use std::fs;
use std::i64;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub struct Tape<'a> {
	pub idx: Value,
	pub tape: FxHashMap<Value, Value>,
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
	pub fn new(root: Option<&'a Path>) -> Tape<'a> {
		Tape {
			idx: Value::I(0),
			dir: true,
			tape: FxHashMap::default(),
			root: root,
		}
	}
	pub fn step(&mut self) {
		self.idx.advance(self.dir)
	}
	pub fn read_val(&self, i: &Value) -> Value {
		if let Some(x) = self.tape.get(&i) {
			x.clone()
		} else {
			Value::I(0)
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
			Entry::Vacant(ent) => {
				ent.insert(Value::I(1));
			}
		}
	}
	pub fn op9(&mut self) {
		self.step();
		let a = self.read_int();
		match self.tape.entry(a) {
			Entry::Occupied(mut ent) => ent.get_mut().decr(),
			Entry::Vacant(ent) => {
				ent.insert(Value::I(-1));
			}
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
		let a = self.read_val(&a);
		let (aiter, alen) = ValueAsChars::new(&a);
		self.step();
		let mut b = self.read_int();
		self.tape.insert(b.clone(), Value::I(alen as i64));
		for ch in aiter {
			b.advance(self.dir);
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
					a.advance(self.dir);
				}
				self.tape.insert(c, Value::from(s));
			}
			_ => self.step(),
		}
	}

	fn mk_child<'b>(&'b mut self, path: Option<&'b Path>, oi: Value, ii: Value) -> TapeChild<'b, 'a>
	where
		'a: 'b,
	{
		TapeChild::<'b, 'a> {
			tape: Tape::<'b>::new(path),
			parent: self,
			oidx: oi,
			iidx: ii,
		}
	}

	pub fn op14(
		&mut self,
		stdlib: &FxHashMap<&'static str, FxHashMap<Value, Value>>,
		modcache: &mut FxHashMap<PathBuf, FxHashMap<Value, Value>>,
	) {
		self.step();
		let pathidx = self.idx.clone();
		self.step();
		let oi = self.read_int();
		self.step();
		let ii = self.read_int();
		let cachetape;
		let path = {
			let path = match self.read_val(&pathidx) {
				Value::S(ref x) => {
					let fpath = Path::new(&x[..]);
					let (path, is_file) = if let Some(root) = self.root {
						let path = root.join(fpath);
						let is_file = path.is_file();
						(path, is_file)
					} else {
						(PathBuf::new(), false)
					};
					if !is_file {
						if let Some(lib) = stdlib.get(&x[..]).map(|m| m.clone()) {
							let mut child = self.mk_child(None, oi, ii);
							child.tape.tape = lib;
							child.run(stdlib, modcache);
						}
						return;
					}
					path
				}
				Value::I(x) => {
					let xs = x.to_string();
					let fpath = Path::new(&xs);
					if let Some(root) = self.root {
						root.join(fpath)
					} else {
						return;
					}
				}
				Value::C(x) => {
					let mut buf = [0u8; 4];
					let cx = x.encode_utf8(&mut buf);
					let fpath = Path::new(cx);
					if let Some(root) = self.root {
						root.join(fpath)
					} else {
						return;
					}
				}
			};
			let mut child = self.mk_child(path.parent(), oi, ii);
			if let Some(m) = modcache.get(&path).map(|m| m.clone()) {
				child.tape.tape = m;
				child.run(stdlib, modcache);
				return;
			} else if let Ok(f) = fs::File::open(&path) {
				let mut f = BufReader::new(f);
				let mut line = String::new();
				let mut idx = 0;
				while let Ok(n) = f.read_line(&mut line) {
					if n == 0 {
						break;
					}
					child
						.tape
						.tape
						.insert(Value::I(idx), Value::from(line.trim_end_matches('\n')));
					line.clear();
					idx += 1;
				}
				cachetape = child.tape.tape.clone();
				child.run(stdlib, modcache);
				path.to_owned()
			} else {
				return;
			}
		};
		modcache.insert(path, cachetape);
	}
	pub fn op15(&mut self) {
		self.step();
		let a = self.read_int();
		match self.tape.entry(a) {
			Entry::Occupied(mut ent) => {
				let val = ent.get_mut();
				let mut rng = thread_rng();
				match *val {
					Value::I(ref mut x @ i64::MAX) => *x = rng.gen_range(0..=i64::MAX),
					Value::I(ref mut x) => {
						if *x > 0 {
							*x = rng.gen_range(0..=*x)
						}
					}
					Value::S(ref mut x) if num_gtz(x) => {
						let range9 = uniform::Uniform::new_inclusive(b'0', b'9');
						let s = Rc::make_mut(x);
						let b = unsafe { s.as_mut_vec() };
						let mut oldb = b.clone();
						while {
							for c in b.iter_mut() {
								*c = range9.sample(&mut rng);
							}
							b.cmp(&&mut oldb) == Ordering::Greater
						} {}
						while b[0] == b'0' {
							b.swap_remove(0);
						}
					}
					Value::S(ref x) if is_num(x) => (),
					_ => *val = Value::I(0),
				}
			}
			Entry::Vacant(_) => (),
		}
	}
	pub fn op16(&mut self) {
		self.step();
		let a = self.read_int();
		let a = self.read_val(&a);
		let (aiter, alen) = ValueAsChars::new(&a);
		self.step();
		let mut b = self.read_int();
		self.tape.insert(b.clone(), Value::I(alen as i64));
		for ch in aiter {
			b.advance(self.dir);
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
						Value::I(x) if x >= 0 && x <= 0x10ffff => {
							char::from_u32(x as u32).unwrap_or('\u{fffd}')
						}
						_ => '\u{fffd}',
					});
					a.advance(self.dir);
				}
				self.tape.insert(c, Value::from(s));
			}
			_ => self.step(),
		}
	}
	pub fn run(&mut self) {
		let mut modcache = FxHashMap::default();
		let stdlib = gen_libs();
		loop {
			match self.tape.get(&self.idx) {
				Some(&Value::I(cell)) => match cell {
					1 => self.op1(),
					2 => self.dir ^= true,
					3 => return,
					4 => {
						self.step();
						let a = self.read_int();
						print!("{}", self.read_val(&a));
					}
					5 => {
						(&mut io::stdout() as &mut dyn io::Write).flush().ok();
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
					}
					6 => {
						self.step();
						self.idx = self.read_int();
						continue;
					}
					7 => {
						self.op7();
						continue;
					}
					8 => self.op8(),
					9 => self.op9(),
					10 => {
						self.op10();
						continue;
					}
					11 => println!(""),
					12 => self.op12(),
					13 => self.op13(),
					14 => self.op14(&stdlib, &mut modcache),
					15 => self.op15(),
					16 => self.op16(),
					17 => self.op17(),
					_ => (),
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
	pub fn run(
		&mut self,
		stdlib: &FxHashMap<&'static str, FxHashMap<Value, Value>>,
		modcache: &mut FxHashMap<PathBuf, FxHashMap<Value, Value>>,
	) {
		loop {
			match self.tape.tape.get(&self.tape.idx) {
				Some(&Value::I(cell)) => match cell {
					1 => self.tape.op1(),
					2 => self.tape.dir ^= true,
					3 => return,
					4 => {
						self.step();
						let a = self.read_int();
						let a = self.read_val(&a);
						self.parent.tape.insert(self.oidx.clone(), a);
						self.oidx.advance(self.parent.dir);
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
						self.iidx.advance(self.parent.dir);
					}
					6 => {
						self.step();
						self.tape.idx = self.read_int();
						continue;
					}
					7 => {
						self.tape.op7();
						continue;
					}
					8 => self.tape.op8(),
					9 => self.tape.op9(),
					10 => {
						self.tape.op10();
						continue;
					}
					12 => self.tape.op12(),
					13 => self.tape.op13(),
					14 => self.tape.op14(stdlib, modcache),
					15 => self.tape.op15(),
					16 => self.tape.op16(),
					17 => self.tape.op17(),
					_ => (),
				},
				Some(_) => (),
				_ => return,
			}
			self.step();
		}
	}
}
