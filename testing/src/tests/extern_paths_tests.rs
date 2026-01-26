use super::*;

proto_package!(EXTERN_PATH_TEST, name = "extern_path_test", no_cel_test);

define_proto_file!(
  FILE,
  name = "file.proto",
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

pub mod submod {
  use super::*;

  use_proto_file!(FILE, extern_path = "testing::submod");

  #[proto_message]
  pub struct SubmodMsg {
    pub id: i32,
  }

  #[proto_message]
  #[proto(parent_message = SubmodMsg)]
  pub struct SubmodNestedMsg {
    pub id: i32,
  }

  #[proto_enum]
  pub enum SubmodEnum {
    Unspecified,
    A,
    B,
  }

  #[proto_enum]
  #[proto(parent_message = SubmodMsg)]
  pub enum SubmodNestedEnum {
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
      ".extern_path_test.SubmodMsg",
      "::testing::submod::SubmodMsg",
    ),
    (
      ".extern_path_test.SubmodMsg.SubmodNestedMsg",
      "::testing::submod::SubmodNestedMsg",
    ),
    (
      ".extern_path_test.SubmodEnum",
      "::testing::submod::SubmodEnum",
    ),
    (
      ".extern_path_test.SubmodMsg.SubmodNestedEnum",
      "::testing::submod::SubmodNestedEnum",
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
