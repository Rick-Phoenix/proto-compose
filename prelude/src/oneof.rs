use crate::*;

pub trait ProtoOneof {
  fn proto_schema() -> Oneof;

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
