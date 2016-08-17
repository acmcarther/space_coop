use glutin::VirtualKeyCode;

// Add keys here as needed
pub fn str_to_virtual_key_code(c: &str) -> Option<VirtualKeyCode> {
  use glutin::VirtualKeyCode::*;
  match c {
    "a" | "A" => Some(A),
    "w" | "W" => Some(W),
    "s" | "S" => Some(S),
    "d" | "D" => Some(D),
    "esc" | "ESC" => Some(Escape),
    _ => None,
  }
}
