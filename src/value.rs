use std::cmp::{self, Ord, Ordering};
use std::fmt::{self, Display};
use std::i64;
use std::mem::MaybeUninit;
use std::rc::Rc;
use std::str::Chars;

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum Value {
	S(Rc<String>),
	I(i64),
	C(char),
}

pub enum ValueAsChars<'a> {
	S(Chars<'a>),
	I([MaybeUninit<u8>; 21]),
	C(char),
	None,
}

impl<'a> ValueAsChars<'a> {
	pub fn new(val: &'a Value) -> (ValueAsChars, usize) {
		match *val {
			Value::I(i64::MIN) => (
				ValueAsChars::I([
					MaybeUninit::new(20),
					MaybeUninit::new(b'8'),
					MaybeUninit::new(b'0'),
					MaybeUninit::new(b'8'),
					MaybeUninit::new(b'5'),
					MaybeUninit::new(b'7'),
					MaybeUninit::new(b'7'),
					MaybeUninit::new(b'4'),
					MaybeUninit::new(b'7'),
					MaybeUninit::new(b'8'),
					MaybeUninit::new(b'6'),
					MaybeUninit::new(b'3'),
					MaybeUninit::new(b'0'),
					MaybeUninit::new(b'2'),
					MaybeUninit::new(b'7'),
					MaybeUninit::new(b'3'),
					MaybeUninit::new(b'3'),
					MaybeUninit::new(b'2'),
					MaybeUninit::new(b'2'),
					MaybeUninit::new(b'9'),
					MaybeUninit::new(b'-'),
				]),
				20,
			),
			Value::I(0) => (ValueAsChars::C('0'), 1),
			Value::I(mut x) => {
				let mut buf: [MaybeUninit<u8>; 21] = [
					MaybeUninit::uninit(),
					MaybeUninit::uninit(),
					MaybeUninit::uninit(),
					MaybeUninit::uninit(),
					MaybeUninit::uninit(),
					MaybeUninit::uninit(),
					MaybeUninit::uninit(),
					MaybeUninit::uninit(),
					MaybeUninit::uninit(),
					MaybeUninit::uninit(),
					MaybeUninit::uninit(),
					MaybeUninit::uninit(),
					MaybeUninit::uninit(),
					MaybeUninit::uninit(),
					MaybeUninit::uninit(),
					MaybeUninit::uninit(),
					MaybeUninit::uninit(),
					MaybeUninit::uninit(),
					MaybeUninit::uninit(),
					MaybeUninit::uninit(),
					MaybeUninit::uninit(),
				];
				let neg = x < 0;
				if neg {
					x = -x
				};
				let mut xlen = 1;
				while {
					unsafe { *buf.get_unchecked_mut(xlen).as_mut_ptr() = b'0' + (x % 10) as u8 };
					x /= 10;
					x != 0
				} {
					xlen += 1
				}
				if neg {
					xlen += 1;
					unsafe { *buf.get_unchecked_mut(xlen).as_mut_ptr() = b'-' };
				}
				unsafe { *buf[0].as_mut_ptr() = xlen as u8 };
				(ValueAsChars::I(buf), xlen)
			}
			Value::S(ref s) => (ValueAsChars::S(s.chars()), s.chars().count()),
			Value::C(c) => (ValueAsChars::C(c), 1),
		}
	}
}

impl<'a> Iterator for ValueAsChars<'a> {
	type Item = char;
	fn next(&mut self) -> Option<char> {
		match *self {
			ValueAsChars::S(ref mut chs) => chs.next(),
			ValueAsChars::I(ref mut xs) => {
				let idx = unsafe { xs[0].assume_init() } as usize;
				if idx == 0 {
					None
				} else {
					unsafe {
						let x = xs.get_unchecked(idx).assume_init();
						*xs[0].as_mut_ptr() = (idx - 1) as u8;
						Some(x as char)
					}
				}
			}
			ValueAsChars::C(c) => {
				*self = ValueAsChars::None;
				Some(c)
			}
			ValueAsChars::None => None,
		}
	}
}

impl Display for Value {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Value::S(ref x) => write!(f, "{}", x),
			Value::I(x) => write!(f, "{}", x),
			Value::C(x) => write!(f, "{}", x),
		}
	}
}

fn num_decr_core(s: &mut Vec<u8>, start: usize) {
	for n in s[start..].iter_mut().rev() {
		if *n == b'0' {
			*n = b'9';
		} else {
			*n -= 1;
			break;
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
			return;
		}
	}
	s.insert(start, b'1');
}

fn num_incr_by_core(x: &[u8], y: &[u8], n: bool) -> Value {
	let (xlen, ylen) = (x.len(), y.len());
	let xylen = cmp::max(xlen, ylen);
	let mut z = Vec::with_capacity(xylen + 2);
	let mut carry = 0;
	for i in 0..xylen {
		let xc = if i >= xlen - 1 {
			0
		} else {
			x[xlen - i - 1] - b'0'
		};
		let yc = if i >= ylen - 1 {
			0
		} else {
			x[ylen - i - 1] - b'0'
		};
		let n = xc + yc + carry;
		z.push(if n > 10 {
			carry = 1;
			n - (b'0' + 10)
		} else {
			n - b'0'
		});
	}
	if carry == 1 {
		z.push(b'1');
	}
	if n {
		z.push(b'-');
	}
	z.reverse();
	Value::from(unsafe { String::from_utf8_unchecked(z) })
}

fn num_decr_by_core(x: &[u8], y: &[u8], n: bool) -> Value {
	let (xlen, ylen) = (x.len(), y.len());
	let xylen = cmp::max(xlen, ylen);
	let mut z = Vec::with_capacity(xylen);
	let mut carry = 0;
	for i in 0..xylen {
		let xc = if i >= xlen - 1 {
			0
		} else {
			x[xlen - i - 1] - b'0'
		};
		let yc = if i >= ylen - 1 {
			0
		} else {
			x[ylen - i - 1] - b'0'
		} + carry;
		if yc > xc {
			z.push((b'0' + 10) - (yc - xc));
			carry = 1;
		} else {
			z.push(b'0' + xc - yc);
			carry = 0;
		}
	}
	if n {
		z.push(b'-');
	}
	z.reverse();
	Value::from(unsafe { String::from_utf8_unchecked(z) })
}

fn unum_cmp(a: &[u8], b: &[u8]) -> Ordering {
	let alen = a.len();
	let blen = b.len();
	if alen > blen {
		Ordering::Greater
	} else if blen > alen {
		Ordering::Less
	} else {
		a.cmp(b)
	}
}

pub fn num_gtz(s: &str) -> bool {
	let mut chs = s.bytes();
	match chs.next() {
		Some(b'1'..=b'9') => chs.all(|c| c >= b'0' && c <= b'9'),
		_ => false,
	}
}

pub fn is_num(s: &str) -> bool {
	s == "0" || {
		let mut chs = s.bytes();
		match chs.next() {
			Some(b'-') => match chs.next() {
				Some(b'1'..=b'9') => chs.all(|c| c >= b'0' && c <= b'9'),
				_ => false,
			},
			Some(b'1'..=b'9') => chs.all(|c| c >= b'0' && c <= b'9'),
			_ => false,
		}
	}
}

fn i64_parse(s: &str) -> Option<i64> {
	if s == "0" {
		return Some(0);
	}
	let mut chs = s.bytes();
	let mut first = chs.next();
	let neg = first == Some(b'-');
	if neg {
		first = chs.next();
	}
	let mut val = match first {
		Some(x @ b'1'..=b'9') => (x - b'0') as u64,
		_ => return None,
	};
	for c in chs {
		match c {
			x @ b'0'..=b'9' => {
				if let Some(v10) = val
					.checked_mul(10)
					.and_then(move |v10| v10.checked_add((x - b'0') as u64))
				{
					val = v10;
				} else {
					return None;
				}
			}
			_ => return None,
		}
	}
	if neg {
		if val > i64::MAX as u64 + 1 {
			None
		} else if val == i64::MAX as u64 + 1 {
			Some(i64::MIN)
		} else {
			Some(-(val as i64))
		}
	} else if val > i64::MAX as u64 {
		None
	} else {
		Some(val as i64)
	}
}

impl Value {
	pub fn advance(&mut self, direction: bool) {
		if direction {
			self.incr()
		} else {
			self.decr()
		}
	}

	pub fn incr(&mut self) {
		let newx;
		loop {
			match *self {
				Value::I(i64::MAX) => {
					*self = Value::S(Rc::new(String::from("9223372036854775808")))
				}
				Value::I(ref mut x) => *x += 1,
				Value::S(ref mut x) => {
					if is_num(&x[..]) {
						let s = Rc::make_mut(x);
						unsafe {
							let s = s.as_mut_vec();
							if s[0] == b'-' {
								num_decr_core(s, 1)
							} else {
								num_incr_core(s, 0)
							}
						}
						if let Some(x) = i64_parse(s) {
							newx = x;
							break;
						}
					} else {
						newx = 1;
						break;
					}
				}
				Value::C(_) => *self = Value::I(1),
			}
			return;
		}
		*self = Value::I(newx);
	}

	pub fn decr(&mut self) {
		let newx;
		loop {
			match *self {
				Value::I(i64::MIN) => {
					*self = Value::S(Rc::new(String::from("-9223372036854775809")))
				}
				Value::I(ref mut x) => *x -= 1,
				Value::S(ref mut x) => {
					if is_num(&x[..]) {
						let s = Rc::make_mut(x);
						unsafe {
							let s = s.as_mut_vec();
							if s[0] == b'-' {
								num_incr_core(s, 1)
							} else {
								num_decr_core(s, 0)
							}
						}
						if let Some(x) = i64_parse(s) {
							newx = x;
							break;
						}
					} else {
						newx = -1;
						break;
					}
				}
				Value::C(_) => *self = Value::I(-1),
			}
			return;
		}
		*self = Value::I(newx);
	}

	pub fn incr_by(&self, rhs: &Value) -> Value {
		match (self, rhs) {
			(&Value::I(x), &Value::I(y)) => {
				if let Some(z) = x.checked_add(y) {
					return Value::I(z);
				}
			}
			(_, &Value::S(ref s)) if !is_num(s) => return self.clone(),
			(_, &Value::I(0)) | (_, &Value::C(_)) => return self.clone(),
			(&Value::S(ref s), _) if !is_num(s) => return rhs.clone(),
			(&Value::I(0), _) | (&Value::C(_), _) => return rhs.clone(),
			_ => (),
		}
		let (xs, ys) = (self.to_string(), rhs.to_string());
		let (x, y) = (xs.as_bytes(), ys.as_bytes());
		let (xn, yn) = (x[0] == b'-', y[0] == b'-');
		if xn == yn {
			if xn {
				num_incr_by_core(&x[1..], &y[1..], true)
			} else {
				num_incr_by_core(x, y, false)
			}
		} else {
			if xn {
				match unum_cmp(&x[1..], y) {
					Ordering::Equal => Value::I(0),
					Ordering::Less => num_decr_by_core(y, &x[1..], false),
					Ordering::Greater => num_decr_by_core(&x[1..], y, true),
				}
			} else {
				match unum_cmp(x, &y[1..]) {
					Ordering::Equal => Value::I(0),
					Ordering::Less => num_decr_by_core(&y[1..], x, true),
					Ordering::Greater => num_decr_by_core(x, &y[1..], false),
				}
			}
		}
	}

	pub fn decr_by(&self, rhs: &Value) -> Value {
		match (self, rhs) {
			(&Value::I(x), &Value::I(y)) => {
				if let Some(z) = x.checked_sub(y) {
					return Value::I(z);
				}
			}
			(_, &Value::S(ref s)) if !is_num(s) => return self.clone(),
			(_, &Value::I(0)) | (_, &Value::C(_)) => return self.clone(),
			(&Value::S(ref s), _) if !is_num(s) => return rhs.as_negative_unchecked(),
			(&Value::I(0), _) | (&Value::C(_), _) => return rhs.as_negative_unchecked(),
			_ => (),
		}
		let (xs, ys) = (self.to_string(), rhs.to_string());
		let (x, y) = (xs.as_bytes(), ys.as_bytes());
		let (xn, yn) = (x[0] == b'-', y[0] == b'-');
		if xn != yn {
			if xn {
				num_incr_by_core(&x[1..], y, true)
			} else {
				num_incr_by_core(x, &y[1..], false)
			}
		} else {
			if xn {
				match unum_cmp(&x[1..], y) {
					Ordering::Equal => Value::I(0),
					Ordering::Less => num_decr_by_core(y, &x[1..], false),
					Ordering::Greater => num_decr_by_core(&x[1..], y, true),
				}
			} else {
				match unum_cmp(x, &y[1..]) {
					Ordering::Equal => Value::I(0),
					Ordering::Less => num_decr_by_core(&y[1..], x, true),
					Ordering::Greater => num_decr_by_core(x, &y[1..], false),
				}
			}
		}
	}

	pub fn as_negative_unchecked(&self) -> Value {
		match *self {
			Value::I(i64::MIN) => Value::S(Rc::new(String::from("9223372036854775808"))),
			Value::I(x) => Value::I(-x),
			Value::S(ref s) => {
				if s.as_bytes()[0] == b'-' {
					Value::S(Rc::new(String::from(&s[1..])))
				} else if &s[..] == "9223372036854775808" {
					Value::I(i64::MIN)
				} else {
					let mut news = String::with_capacity(s.len() + 1);
					news.push('-');
					news.push_str(&s[..]);
					Value::S(Rc::new(news))
				}
			}
			Value::C(_) => Value::I(0),
		}
	}
}

impl<'a> From<&'a str> for Value {
	fn from(s: &'a str) -> Value {
		if let Some(x) = i64_parse(&s) {
			Value::I(x)
		} else {
			if !s.is_empty() && s.chars().nth(1).is_none() {
				Value::C(s.chars().nth(0).unwrap())
			} else {
				Value::S(Rc::new(String::from(s)))
			}
		}
	}
}

impl From<String> for Value {
	fn from(s: String) -> Value {
		if let Some(x) = i64_parse(&s) {
			Value::I(x)
		} else {
			if !s.is_empty() && s.chars().nth(1).is_none() {
				Value::C(s.chars().nth(0).unwrap())
			} else {
				Value::S(Rc::new(s))
			}
		}
	}
}

impl From<char> for Value {
	fn from(c: char) -> Value {
		match c {
			'0'..='9' => Value::I((c as u32 - '0' as u32) as i64),
			_ => Value::C(c),
		}
	}
}
