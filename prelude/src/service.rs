use crate::*;

pub trait ProtoService {
  fn as_proto_service() -> Service;
}

#[derive(Debug, PartialEq, Template)]
#[template(path = "service.proto.j2")]
pub struct Service {
  pub name: &'static str,
  pub file: &'static str,
  pub options: Vec<ProtoOption>,
  pub handlers: Vec<ServiceHandler>,
  pub package: &'static str,
}

#[derive(Debug, PartialEq)]
pub struct ServiceHandler {
  pub name: &'static str,
  pub options: Vec<ProtoOption>,
  pub request: ProtoPath,
  pub response: ProtoPath,
}
