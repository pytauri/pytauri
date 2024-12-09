use std::env;

fn main() {
	let target_triple = env::var("TARGET").unwrap();
	println!("cargo:rustc-env=TARGET={}", target_triple);
}
