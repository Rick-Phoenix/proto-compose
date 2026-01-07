use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::{env, path::PathBuf};

use prost_build::Config;
use prost_reflect::{prost::Message, prost_types::FileDescriptorSet};

pub fn set_up_validators(
  config: &mut Config,
  files: &[impl AsRef<Path>],
  include_paths: &[impl AsRef<Path>],
  packages: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
  let out_dir = env::var("OUT_DIR")
    .map(PathBuf::from)
    .unwrap_or(env::temp_dir());

  config
    .extern_path(".google.protobuf", "::proto_types")
    .extern_path(".buf.validate", "::proto_types::protovalidate")
    .compile_well_known_types();

  let temp_descriptor_path = out_dir.join("temp_file_descriptor_set.bin");
  {
    let mut temp_config = prost_build::Config::new();
    temp_config.file_descriptor_set_path(&temp_descriptor_path);
    temp_config.out_dir(&out_dir);
    temp_config.compile_protos(files, include_paths)?;
  }

  let mut fds_file = std::fs::File::open(&temp_descriptor_path)?;
  let mut fds_bytes = Vec::new();
  fds_file.read_to_end(&mut fds_bytes)?;
  let fds = FileDescriptorSet::decode(fds_bytes.as_slice())?;
  let pool = prost_reflect::DescriptorPool::from_file_descriptor_set(fds)?;

  for message_desc in pool.all_messages() {
    let package = message_desc.package_name();

    if packages.contains(&package) {
      let message_name = message_desc.full_name();

      config.message_attribute(message_name, "#[derive(::prelude::ValidatedMessage)]");
      #[cfg(feature = "cel")]
      {
        config.message_attribute(message_name, "#[derive(::prelude::CelValue)]");
      }
      config.message_attribute(
        message_name,
        format!(r#"#[proto(name = "{message_name}")]"#),
      );

      for oneof in message_desc.oneofs() {
        let parent_message = oneof.parent_message().full_name();

        config.enum_attribute(oneof.full_name(), "#[derive(::prelude::ValidatedOneof)]");
        #[cfg(feature = "cel")]
        {
          config.enum_attribute(oneof.full_name(), "#[derive(::prelude::CelOneof)]");
        }
        config.enum_attribute(
          oneof.full_name(),
          format!(r#"#[proto(parent_message = "{parent_message}")]"#),
        );
      }
    }
  }

  for enum_desc in pool.all_enums() {
    let package = enum_desc.package_name();

    if packages.contains(&package) {
      let full_ish_name = enum_desc
        .full_name()
        .strip_prefix(&format!("{}.", enum_desc.package_name()))
        .unwrap_or(enum_desc.full_name());

      config.enum_attribute(full_ish_name, "#[derive(::prelude::ProtoEnum)]");
      config.enum_attribute(
        full_ish_name,
        format!(r#"#[proto(name = "{full_ish_name}")]"#),
      );
    }
  }

  Ok(())
}

/// A helper to use when gathering the names of proto files to pass to [`prost_build::Config::compile_protos`].
/// Recursively collects all .proto files in a given directory and its subdirectories.
pub fn get_proto_files_recursive(base_dir: impl Into<PathBuf>) -> io::Result<Vec<String>> {
  let base_dir: PathBuf = base_dir.into();
  let mut proto_files = Vec::new();

  if !base_dir.is_dir() {
    return Err(io::Error::new(
      io::ErrorKind::InvalidInput,
      format!("Path {} is not a directory.", base_dir.display()),
    ));
  }

  collect_proto_files_recursive_helper(base_dir.as_path(), &mut proto_files)?;

  Ok(proto_files)
}

fn collect_proto_files_recursive_helper(
  current_dir: &Path,
  proto_files: &mut Vec<String>,
) -> io::Result<()> {
  for entry in fs::read_dir(current_dir)? {
    let entry = entry?;
    let path = entry.path();

    if path.is_file() {
      if path.extension().is_some_and(|ext| ext == "proto") {
        proto_files.push(
          path
            .to_str()
            .ok_or_else(|| {
              io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Path {} contains invalid Unicode.", path.display()),
              )
            })?
            .to_owned(),
        );
      }
    } else if path.is_dir() {
      collect_proto_files_recursive_helper(&path, proto_files)?;
    }
  }
  Ok(())
}
