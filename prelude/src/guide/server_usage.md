# Server Usage

The recommended workflow is to define the proto items in a separate workspace crate (which I will refer to as the "models" crate) and export the package handle, so that the consuming crate (like a tonic server) can use the handle to generate the files and to generate the services from those files, while importing the pre-built messages from the models crate.

This is how to set up the `build.rs` file in the consuming crate, which is this case will be a tonic server.

(You can find the most up-to-date example in the `test-server` crate of the repo. I can't really keep this up-to-date as an example since it relies on file generation, so if this should become stale or incorrect, please open an issue or PR).

```rust,ignore

use std::env;

use tonic_prost_build::Config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=../models/src/");

    // We import the package handle from the models crate.
    // (Special considerations needed for no_std crates can be found in the docs)
    let pkg = models::PKG.get_package();

    // Create the proto files, which we need
    // to generate the services
    pkg
        .render_files(concat!(env!("CARGO_MANIFEST_DIR"), "/proto"))
        .unwrap();

    let include_paths = &["proto", "proto_deps"];
    
    let files = &[ "file1.proto", "and_so_on_and_so_forth.proto" ];

    let mut config = Config::new();

    config
        .extern_path(".google.protobuf", "::proto_types")
        // If we are using validators
        .extern_path(".buf.validate", "::proto_types::protovalidate")
        .compile_well_known_types();

    // We only need to build the services, and we will import
    // the pre-built messages directly from our models crate
    // 
    // We use the `extern_paths` helper from the package so that
    // each entry is automatically mapped
    for (name, path) in pkg.extern_paths() {
        config.extern_path(name, path);
    }

    config.compile_protos(files, include_paths)?;

    tonic_prost_build::configure()
        .compile_with_config(config, files, include_paths)?;

    Ok(())
}
```
