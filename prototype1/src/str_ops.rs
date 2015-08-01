pub use self::str_ops::{
  default_string,
};

mod str_ops {
  use std::ascii::AsciiExt;

  pub fn default_string(string: &str, default: &str) -> String {
    if string == "" {
      default.to_string()
    } else {
      string.to_string()
    }
  }
}
