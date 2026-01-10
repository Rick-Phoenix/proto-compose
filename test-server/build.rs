// use tonic_prost_build::Config;
//
fn main() -> Result<(), Box<dyn std::error::Error>> {
  //   println!("cargo:rerun-if-changed=../testing/proto_test/");
  //
  //   let package = testing::MYAPP_V1.get_package();
  //
  //   package.render_files("../testing/proto_test/")?;
  //
  //   let mut config = Config::new();
  //
  //   config
  //     .extern_path(".google.protobuf", "::proto_types")
  //     .extern_path(".buf.validate", "::proto_types::protovalidate")
  //     .compile_well_known_types()
  //     .bytes(["."]);
  //
  //   for (item, path) in package.extern_paths() {
  //     config.extern_path(&item, &path);
  //   }
  //
  //   let proto_include_paths = &["../testing/proto_test/", "proto_deps"];
  //
  //   let proto_files = &["../testing/proto_test/abc.proto"];
  //
  //   tonic_prost_build::configure()
  //     .build_client(false)
  //     .compile_with_config(config, proto_files, proto_include_paths)?;
  //
  Ok(())
}
