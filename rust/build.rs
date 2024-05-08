extern crate cbindgen;

use std::env;

fn main() {
  let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

  println!("Crate Dir: {}", crate_dir);

  cbindgen::generate(&crate_dir)
    .expect("Unable to generate bindings")
    .write_to_file("target/include/cardano.h");
}
