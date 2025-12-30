use crate::*;

pub trait ProtoOneof {
  fn proto_schema() -> Oneof;

  fn name() -> &'static str;

  fn tags() -> &'static [i32];

  fn check_tags(message: &str, found_tags: &mut [i32]) -> Result<(), String> {
    use similar_asserts::SimpleDiff;

    let expected = Self::tags();
    let oneof_name = Self::name();

    found_tags.sort_unstable();

    if expected != found_tags {
      let exp_str = format!("{expected:#?}");
      let found_str = format!("{found_tags:#?}");

      let diff = SimpleDiff::from_str(&exp_str, &found_str, "expected", "found");

      let error =
        format!("Found tags mismatch for oneof {oneof_name} in message {message}:\n{diff}");

      return Err(error);
    }

    Ok(())
  }

  fn validate(
    &self,
    _parent_messages: &mut Vec<FieldPathElement>,
    _violations: &mut ViolationsAcc,
  ) {
  }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Oneof {
  pub name: &'static str,
  pub fields: Vec<ProtoField>,
  pub options: Vec<ProtoOption>,
}
