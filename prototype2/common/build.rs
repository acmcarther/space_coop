extern crate itertools;
extern crate serde_codegen;

use std::env;
use std::path::Path;

use itertools::Itertools;

pub fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();

    (vec!["world.rs", "protocol.rs"]).into_iter().foreach(|path| {
      let full_src = "src/".to_owned() + path + ".in";
      let src = Path::new(&full_src);
      let dst = Path::new(&out_dir).join(path);
      serde_codegen::expand(&src, &dst).unwrap();
    })
}
