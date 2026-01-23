use proto_types::{Any, Code, Duration, FieldMask, Status, Timestamp};

use crate::*;

impl AsProtoType for Duration {
  fn proto_type() -> ProtoType {
    ProtoType::Message(ProtoPath {
      name: "Duration".into(),
      package: "google.protobuf".into(),
      file: "google/protobuf/duration.proto".into(),
    })
  }
}

impl AsProtoType for Timestamp {
  fn proto_type() -> ProtoType {
    ProtoType::Message(ProtoPath {
      name: "Timestamp".into(),
      package: "google.protobuf".into(),
      file: "google/protobuf/timestamp.proto".into(),
    })
  }
}

impl AsProtoType for Any {
  fn proto_type() -> ProtoType {
    ProtoType::Message(ProtoPath {
      name: "Any".into(),
      package: "google.protobuf".into(),
      file: "google/protobuf/any.proto".into(),
    })
  }
}

impl AsProtoType for () {
  fn proto_type() -> ProtoType {
    ProtoType::Message(ProtoPath {
      name: "Empty".into(),
      package: "google.protobuf".into(),
      file: "google/protobuf/empty.proto".into(),
    })
  }
}

impl AsProtoType for FieldMask {
  fn proto_type() -> ProtoType {
    ProtoType::Message(ProtoPath {
      name: "FieldMask".into(),
      package: "google.protobuf".into(),
      file: "google/protobuf/field_mask.proto".into(),
    })
  }
}

impl AsProtoType for Status {
  fn proto_type() -> ProtoType {
    ProtoType::Message(ProtoPath {
      name: "Status".into(),
      package: "google.rpc".into(),
      file: "google/rpc/status.proto".into(),
    })
  }
}

impl AsProtoType for Code {
  fn proto_type() -> ProtoType {
    ProtoType::Message(ProtoPath {
      name: "Code".into(),
      package: "google.rpc".into(),
      file: "google/rpc/code.proto".into(),
    })
  }
}
