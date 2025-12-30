use testing::collect_package;

fn main() {
  let pkg = collect_package();

  pkg.render_files("proto_test").unwrap();
}

#[test]
fn name() {}
