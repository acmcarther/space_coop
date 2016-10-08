extern crate itertools;
extern crate serde_codegen;


use itertools::Itertools;
use std::env;
use std::path::Path;

pub fn main() {
  let out_dir = env::var_os("OUT_DIR").unwrap();

  (vec!["aspects.rs"]).into_iter().foreach(|path| {
    let full_src = "src/".to_owned() + path + ".in";
    let src = Path::new(&full_src);
    let dst = Path::new(&out_dir).join(path);
    serde_codegen::expand(&src, &dst).unwrap();
  })
}
