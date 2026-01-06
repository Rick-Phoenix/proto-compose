use std::io::Read;
use std::{env, path::PathBuf};

use prost_build::Config;
use prost_reflect::{prost::Message, prost_types::FileDescriptorSet};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  println!("cargo:rerun-if-changed=../proto");

  let out_dir = env::var("OUT_DIR")
    .map(PathBuf::from)
    .unwrap_or(env::temp_dir());
  let descriptor_path = out_dir.join("file_descriptor_set.bin");

  let proto_include_paths = &["proto", "proto_deps"];

  let files = &["proto/reflection.proto"];

  let mut config = Config::new();
  config
    .file_descriptor_set_path(&descriptor_path)
    .extern_path(".google.protobuf", "::proto_types")
    .extern_path(".google.type", "::proto_types")
    .extern_path(".google.rpc", "::proto_types")
    .extern_path(".buf.validate", "::proto_types::protovalidate")
    .compile_well_known_types()
    .bytes(["."])
    .out_dir(&out_dir);

  let temp_descriptor_path = out_dir.join("temp_file_descriptor_set_for_protocheck.bin");
  {
    let mut temp_config = prost_build::Config::new();
    temp_config.file_descriptor_set_path(&temp_descriptor_path);
    temp_config.out_dir(&out_dir);
    temp_config.compile_protos(files, proto_include_paths)?;
  }

  let mut fds_file = std::fs::File::open(&temp_descriptor_path)?;
  let mut fds_bytes = Vec::new();
  fds_file.read_to_end(&mut fds_bytes)?;
  let fds = FileDescriptorSet::decode(fds_bytes.as_slice())?;
  let pool = prost_reflect::DescriptorPool::from_file_descriptor_set(fds)?;

  let packages = ["reflection.v1"];

  for message_desc in pool.all_messages() {
    let message_name = message_desc.full_name();

    if packages.contains(&message_desc.package_name()) {
      config.message_attribute(
        message_name,
        "#[derive(::prelude::ValidatedMessage, ::prelude::TryIntoCelValue)]",
      );
      config.message_attribute(
        message_name,
        "#[cel(cel_crate = ::prelude::cel, proto_types_crate = ::prelude::proto_types)]",
      );
      config.message_attribute(
        message_name,
        format!(r#"#[proto(name = "{message_name}")]"#),
      );

      for oneof in message_desc.oneofs() {
        let oneof_name = oneof.full_name();
        //
      }
    }
  }

  for enum_desc in pool.all_enums() {
    let full_ish_name = enum_desc
      .full_name()
      .strip_prefix(&format!("{}.", enum_desc.package_name()))
      .unwrap_or(enum_desc.full_name());

    config.enum_attribute(full_ish_name, "#[derive(::prelude::NamedEnum)]");
    config.enum_attribute(
      full_ish_name,
      format!(r#"#[proto(name = "{full_ish_name}")]"#),
    );
  }

  config.compile_protos(files, proto_include_paths)?;

  println!(
    "cargo:rustc-env=PROTO_DESCRIPTOR_SET={}",
    descriptor_path.display()
  );

  Ok(())
}
