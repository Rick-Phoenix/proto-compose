use crate::*;

pub trait ProtoOneof {
  fn proto_schema() -> Oneof;

  fn name() -> &'static str;

  fn tags() -> &'static [i32];

  #[cfg(feature = "testing")]
  #[track_caller]
  fn check_tags(message: &str, found_tags: &mut [i32]) {
    let expected = Self::tags();
    let oneof_name = Self::name();

    found_tags.sort_unstable();

    pretty_assertions::assert_eq!(
      found_tags,
      expected,
      "Found tags mismatch for oneof {oneof_name} in message {message}"
    );
  }

  fn validate(&self, _parent_messages: &mut Vec<FieldPathElement>) -> Result<(), Violations> {
    Ok(())
  }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Oneof {
  pub name: &'static str,
  pub fields: Vec<ProtoField>,
  pub options: Vec<ProtoOption>,
}
