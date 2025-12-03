use proto_types::{Any, Code, Duration, FieldMask, Status, Timestamp};

use crate::*;

impl AsProtoType for Duration {
  fn proto_type() -> ProtoType {
    ProtoType::Single(TypeInfo {
      name: "Duration",
      path: Some(ProtoPath {
        package: "google.protobuf",
        file: "google/protobuf/duration.proto",
      }),
    })
  }
}

impl AsProtoType for Timestamp {
  fn proto_type() -> ProtoType {
    ProtoType::Single(TypeInfo {
      name: "Timestamp",
      path: Some(ProtoPath {
        package: "google.protobuf",
        file: "google/protobuf/timestamp.proto",
      }),
    })
  }
}

impl AsProtoType for Any {
  fn proto_type() -> ProtoType {
    ProtoType::Single(TypeInfo {
      name: "Any",
      path: Some(ProtoPath {
        package: "google.protobuf",
        file: "google/protobuf/any.proto",
      }),
    })
  }
}

impl AsProtoType for () {
  fn proto_type() -> ProtoType {
    ProtoType::Single(TypeInfo {
      name: "Empty",
      path: Some(ProtoPath {
        package: "google.protobuf",
        file: "google/protobuf/empty.proto",
      }),
    })
  }
}

impl AsProtoType for FieldMask {
  fn proto_type() -> ProtoType {
    ProtoType::Single(TypeInfo {
      name: "FieldMask",
      path: Some(ProtoPath {
        package: "google.protobuf",
        file: "google/protobuf/field_mask.proto",
      }),
    })
  }
}

impl AsProtoType for Status {
  fn proto_type() -> ProtoType {
    ProtoType::Single(TypeInfo {
      name: "Status",
      path: Some(ProtoPath {
        package: "google.rpc",
        file: "google/rpc/status.proto",
      }),
    })
  }
}

impl AsProtoType for Code {
  fn proto_type() -> ProtoType {
    ProtoType::Single(TypeInfo {
      name: "Code",
      path: Some(ProtoPath {
        package: "google.rpc",
        file: "google/rpc/code.proto",
      }),
    })
  }
}
