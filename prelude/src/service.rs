use crate::*;

pub trait ProtoService {
  fn as_proto_service() -> Service;
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "std", derive(Template))]
#[cfg_attr(feature = "std", template(path = "service.proto.j2"))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Service {
  pub name: FixedStr,
  pub file: FixedStr,
  pub options: Vec<ProtoOption>,
  pub handlers: Vec<ServiceHandler>,
  pub package: FixedStr,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ServiceHandler {
  pub name: FixedStr,
  pub options: Vec<ProtoOption>,
  pub request: ProtoPath,
  pub response: ProtoPath,
}
