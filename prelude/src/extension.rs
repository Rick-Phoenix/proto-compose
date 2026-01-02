use crate::*;

#[derive(Debug, PartialEq)]
pub struct Extension {
  pub target: &'static str,
  pub fields: Vec<ProtoField>,
}

pub trait ProtoExtension {
  fn as_proto_extension() -> Extension;
}
