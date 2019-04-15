extern crate cbindgen;
extern crate cc;

use std::env;

fn main() {
    let crate_dir = String::from("data_storage_api");

    cbindgen::Builder::new()
      .with_crate(crate_dir)
      .with_language(cbindgen::Language::C)
      .generate()
      .expect("Unable to generate bindings") 
      .write_to_file("src/c_rust_test.h");
    cc::Build::new()
        .file("src/c_rust_test.c")
        .compile("crusttest");
}
