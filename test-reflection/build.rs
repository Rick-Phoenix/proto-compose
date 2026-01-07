use std::{env, path::PathBuf};

use builder::set_up_validators;
use prost_build::Config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  println!("cargo:rerun-if-changed=../proto");

  let out_dir = env::var("OUT_DIR")
    .map(PathBuf::from)
    .unwrap_or(env::temp_dir());
  let descriptor_path = out_dir.join("file_descriptor_set.bin");

  let include_paths = &["proto", "proto_deps"];

  let files = &["proto/test_schemas.proto"];

  let mut config = Config::new();
  config
    .file_descriptor_set_path(&descriptor_path)
    .extern_path(".google.type", "::proto_types")
    .extern_path(".google.rpc", "::proto_types")
    .bytes(["."])
    .out_dir(&out_dir);

  set_up_validators(&mut config, files, include_paths, &["test_schemas.v1"])?;

  config.compile_protos(files, include_paths)?;

  println!(
    "cargo:rustc-env=PROTO_DESCRIPTOR_SET={}",
    descriptor_path.display()
  );

  Ok(())
}
