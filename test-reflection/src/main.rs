mod proto {
  include!(concat!(env!("OUT_DIR"), "/reflection.v1.rs"));
}

fn main() {
  println!("Hello, world!");
}
