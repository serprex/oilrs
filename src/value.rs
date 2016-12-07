use std::i64;
use std::fmt::{self, Display};
use std::rc::Rc;

#[derive(Clone, Eq, PartialEq)]
pub enum Value {
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

