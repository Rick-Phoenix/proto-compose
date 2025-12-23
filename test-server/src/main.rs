use testing::inner::Nested2;

pub mod myappv1 {
  tonic::include_proto!("myapp.v1");
}

fn main() {
  let x = Nested2::default();
}
