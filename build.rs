#![feature(stmt_expr_attributes)]
extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
  println!("cargo:rustc-link-search=/usr/lib");

  let mut builder = bindgen::Builder::default();

  #[cfg(feature = "bemenu")]
  builder = bemenu(builder);

  let bindings = builder
    .parse_callbacks(Box::new(bindgen::CargoCallbacks))
    .generate()
    .expect("Unable to generate bindings");

  let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
  bindings
    .write_to_file(out_path.join("bindings.rs"))
    .expect("Couldn't write bindings!");
}

fn bemenu(builder: bindgen::Builder) -> bindgen::Builder {
  println!("cargo:rustc-link-lib=bemenu");
  println!("cargo:rerun-if-changed=bindings/bemenu.h");

  builder.header("bindings/bemenu.h")
}
