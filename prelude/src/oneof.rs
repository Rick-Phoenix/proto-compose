use crate::*;

pub trait IntoOneof: From<Self::Oneof> + Into<Self::Oneof> {
  type Oneof: ProtoOneof + From<Self>;

  fn into_oneof(self) -> Self::Oneof {
    self.into()
  }

  fn from_oneof(oneof: Self::Oneof) -> Self {
    oneof.into()
  }
}

impl<T: IntoOneof> ProtoOneof for T {
  const NAME: &str = T::Oneof::NAME;
  const TAGS: &[i32] = T::Oneof::TAGS;

  fn proto_schema() -> Oneof {
    T::Oneof::proto_schema()
  }
}

pub trait ProtoOneof {
  const NAME: &str;
  const TAGS: &[i32];

  fn proto_schema() -> Oneof;

  #[doc(hidden)]
  fn check_tags(message: &str, found_tags: &mut [i32]) -> Result<(), String> {
    use similar_asserts::SimpleDiff;

    let expected = Self::TAGS;
    let oneof_name = Self::NAME;

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
}

pub trait ValidatedOneof {
  fn validate(&self, parent_messages: &mut Vec<FieldPathElement>, violations: &mut ViolationsAcc);
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Oneof {
  pub name: &'static str,
  pub fields: Vec<ProtoField>,
  pub options: Vec<ProtoOption>,
}
