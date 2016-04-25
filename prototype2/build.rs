extern crate syntex;
extern crate itertools;
extern crate serde_codegen;

use std::env;
use std::path::Path;
use std::fs;
use itertools::Itertools;

pub fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();

    // Don't care if directory already exists
    let _ = fs::create_dir(Path::new(&out_dir).join("client"));
    let _ = fs::create_dir(Path::new(&out_dir).join("common"));
    let _ = fs::create_dir(Path::new(&out_dir).join("server"));
    let _ = fs::create_dir(Path::new(&out_dir).join("server/world"));

    (vec!["common/world.rs", "common/protocol.rs", "server/world/mod.rs"]).into_iter().foreach(|path| {
      let full_src = "src/".to_owned() + path + ".in";
      let src = Path::new(&full_src);
      let dst = Path::new(&out_dir).join(path);
      let mut registry = syntex::Registry::new();
      serde_codegen::register(&mut registry);
      registry.expand("", &src, &dst).unwrap();
    })
}
