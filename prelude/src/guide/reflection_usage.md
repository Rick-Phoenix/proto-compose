# From Proto

The preferred usage for this crate is by defining items in rust, but it is also possible to use the features implemented by protocheck, starting from pre-built proto files.

To do that, you must first add the builder to the build-dependencies.
In this example, we are calling the simple version of the `set_up_validators` function, which simply applies the validators to the specified packages, while not collecting any extra data.
If you need to selectively apply attributes to certain items, you can use the [`DescriptorDataConfig`] struct to make the helper collect the list of oneofs, enums and messages and return it, so that you can use it to selectively apply attributes to some of them.

```rust

use std::{env, path::PathBuf};

use builder::DescriptorDataConfig;
use prost_build::Config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=../proto");

    let out_dir = env::var("OUT_DIR")
        .map(PathBuf::from)
        .unwrap_or(env::temp_dir());

    // Set path for descriptor output
    let descriptor_path = out_dir.join("file_descriptor_set.bin");

    // Your proto files and dependencies
    let include_paths = &["proto", "proto_deps"];

    let files = &["proto/test.proto"];

    let mut config = Config::new();
    config
        .file_descriptor_set_path(&descriptor_path)
        // Required, if bytes fields are used
        .bytes(["."])
        .out_dir(&out_dir);

    let _ = builder::set_up_validators(
        &mut config,
        files,
        include_paths,
        // The packages for which you want to apply the validators
        &["test_schemas.v1"]
    )?;

    // Compile the protos
    config.compile_protos(files, include_paths)?;

    // Emit env for descriptor location
    println!(
        "cargo:rustc-env=PROTO_DESCRIPTOR_SET={}",
        descriptor_path.display()
    );

    Ok(())
}

```
