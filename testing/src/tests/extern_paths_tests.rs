use super::*;

proto_package!(EXTERN_PATH_TEST, name = "extern_path_test", no_cel_test);

define_proto_file!(
  NORMAL,
  name = "normal.proto",
  package = EXTERN_PATH_TEST,
  extern_path = "testing"
);

#[proto_message]
pub struct NormalMsg {
  pub id: i32,
}

#[proto_message]
#[proto(parent_message = NormalMsg)]
pub struct NormalNestedMsg {
  pub id: i32,
}

#[proto_enum]
pub enum NormalEnum {
  Unspecified,
  A,
  B,
}

#[proto_enum]
#[proto(parent_message = NormalMsg)]
pub enum NormalNestedEnum {
  Unspecified,
  A,
  B,
}

pub(crate) mod priv_mod {
  use super::*;

  define_proto_file!(
    RE_EXPORTED,
    name = "re-exported.proto",
    package = EXTERN_PATH_TEST,
    extern_path = "testing"
  );

  #[proto_message]
  pub struct ReExportedMsg {
    pub id: i32,
  }

  #[proto_message]
  #[proto(parent_message = ReExportedMsg)]
  pub struct ReExportedNestedMsg {
    pub id: i32,
  }

  #[proto_enum]
  pub enum ReExportedEnum {
    Unspecified,
    A,
    B,
  }

  #[proto_enum]
  #[proto(parent_message = ReExportedMsg)]
  pub enum ReExportedNestedEnum {
    Unspecified,
    A,
    B,
  }
}

#[test]
fn test_extern_path() {
  let pkg = EXTERN_PATH_TEST.get_package();
  let mut paths = pkg.extern_paths();

  let expected = [
    (
      ".extern_path_test.ReExportedMsg",
      "::testing::ReExportedMsg",
    ),
    (
      ".extern_path_test.ReExportedMsg.ReExportedNestedMsg",
      "::testing::ReExportedNestedMsg",
    ),
    (
      ".extern_path_test.ReExportedEnum",
      "::testing::ReExportedEnum",
    ),
    (
      ".extern_path_test.ReExportedMsg.ReExportedNestedEnum",
      "::testing::ReExportedNestedEnum",
    ),
    (".extern_path_test.NormalMsg", "::testing::NormalMsg"),
    (
      ".extern_path_test.NormalMsg.NormalNestedMsg",
      "::testing::NormalNestedMsg",
    ),
    (".extern_path_test.NormalEnum", "::testing::NormalEnum"),
    (
      ".extern_path_test.NormalMsg.NormalNestedEnum",
      "::testing::NormalNestedEnum",
    ),
  ];

  for (exp_name, exp_path) in expected {
    let idx = paths
      .iter()
      .position(|(name, path)| exp_name == name && exp_path == path)
      .unwrap_or_else(|| panic!("Could not find {exp_name} in the extern paths"));

    paths.remove(idx);
  }

  if !paths.is_empty() {
    panic!("Unexpected extern paths: {paths:#?}");
  }
}
