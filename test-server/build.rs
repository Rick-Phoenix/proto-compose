use std::env;

use builder::set_up_validators;
use tonic_prost_build::Config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  println!("cargo:rerun-if-changed=../test-schemas/src/server_models.rs");

  let pkg = test_schemas::server_models::DB_TEST.get_package();

  pkg
    .render_files(concat!(env!("CARGO_MANIFEST_DIR"), "/proto"))
    .unwrap();

  let include_paths = &["proto", "proto_deps"];

  let files = &["proto/db_test.proto"];

  let mut config = Config::new();

  for (name, path) in pkg.extern_paths() {
    config.extern_path(name, path);
  }

  let _ = set_up_validators(&mut config, files, include_paths, &["db_test"])?;

  config.compile_protos(files, include_paths)?;

  tonic_prost_build::configure()
    .build_client(false)
    .compile_with_config(config, files, include_paths)?;

  Ok(())
}
