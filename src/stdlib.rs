use fnv::FnvHashMap;
use value::Value;

fn parse_lib(src: &str) -> FnvHashMap<Value, Value> {
	let mut lib = FnvHashMap::default();
	let mut idx = 0;
	for line in src.lines() {
		lib.insert(Value::I(idx), Value::from(line));
		idx += 1;
	}
	lib
}

pub fn gen_libs() -> FnvHashMap<&'static str, FnvHashMap<Value, Value>> {
	let mut libs = FnvHashMap::with_capacity_and_hasher(31, Default::default());
	libs.insert("abs", parse_lib(include_str!("lib/abs")));
	libs.insert("add", parse_lib(include_str!("lib/add")));
	libs.insert("call", parse_lib(include_str!("lib/call")));
	libs.insert("commainstr", parse_lib(include_str!("lib/commainstr")));
	libs.insert("div", parse_lib(include_str!("lib/div")));
	libs.insert("division", parse_lib(include_str!("lib/division")));
	libs.insert("echo", parse_lib(include_str!("lib/echo")));
	libs.insert("email", parse_lib(include_str!("lib/email")));
	libs.insert("fibonacci", parse_lib(include_str!("lib/fibonacci")));
	libs.insert("head", parse_lib(include_str!("lib/head")));
	libs.insert("headtail", parse_lib(include_str!("lib/headtail")));
	libs.insert("hello_world", parse_lib(include_str!("lib/hello_world")));
	libs.insert("invert", parse_lib(include_str!("lib/invert")));
	libs.insert("iseq", parse_lib(include_str!("lib/iseq")));
	libs.insert("isnegative", parse_lib(include_str!("lib/isnegative")));
	libs.insert("join", parse_lib(include_str!("lib/join")));
	libs.insert("leq", parse_lib(include_str!("lib/leq")));
	libs.insert("mul", parse_lib(include_str!("lib/mul")));
	libs.insert("quine", parse_lib(include_str!("lib/quine")));
	libs.insert("sleep", parse_lib(include_str!("lib/sleep")));
	libs.insert("splitonce", parse_lib(include_str!("lib/splitonce")));
	libs.insert("startswith", parse_lib(include_str!("lib/startswith")));
	libs.insert("strinstr", parse_lib(include_str!("lib/strinstr")));
	libs.insert("strlen", parse_lib(include_str!("lib/strlen")));
	libs.insert("strsplit", parse_lib(include_str!("lib/strsplit")));
	libs.insert("sub", parse_lib(include_str!("lib/sub")));
	libs.insert("swap", parse_lib(include_str!("lib/swap")));
	libs.insert("trimend", parse_lib(include_str!("lib/trimend")));
	libs.insert("trimstart", parse_lib(include_str!("lib/trimstart")));
	libs.insert("truediv", parse_lib(include_str!("lib/truediv")));
	libs.insert("uniquechars", parse_lib(include_str!("lib/uniquechars")));
	libs
}
