use proto_types::{Any, Code, Duration, Empty, FieldMask, Status, Timestamp};

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

impl AsProtoType for Empty {
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

#[cfg(feature = "common-types")]
mod google_dot_type {
  use super::*;
  use proto_types::*;

  macro_rules! impl_types {
    ($($name:ident),*) => {
      paste! {
        $(
          impl AsProtoType for $name {
            fn proto_type() -> ProtoType {
              ProtoType::Message(ProtoPath {
                name: stringify!($name).into(),
                package: "google.type".into(),
                file: concat!("google/type/", stringify!([ < $name:snake > ]), ".proto").into(),
              })
            }
          }
        )*
      }
    };
  }

  impl_types!(
    Date,
    Interval,
    Money,
    Color,
    Fraction,
    Decimal,
    PostalAddress,
    PhoneNumber,
    Quaternion,
    LocalizedText,
    Expr,
    CalendarPeriod,
    Month
  );

  impl AsProtoType for DayOfWeek {
    fn proto_type() -> ProtoType {
      ProtoType::Message(ProtoPath {
        name: "DayOfWeek".into(),
        package: "google.type".into(),
        file: "google/type/dayofweek.proto".into(),
      })
    }
  }

  impl AsProtoType for LatLng {
    fn proto_type() -> ProtoType {
      ProtoType::Message(ProtoPath {
        name: "LatLng".into(),
        package: "google.type".into(),
        file: "google/type/latlng.proto".into(),
      })
    }
  }

  impl AsProtoType for TimeZone {
    fn proto_type() -> ProtoType {
      ProtoType::Message(ProtoPath {
        name: "TimeZone".into(),
        package: "google.type".into(),
        file: "google/type/datetime.proto".into(),
      })
    }
  }

  impl AsProtoType for TimeOfDay {
    fn proto_type() -> ProtoType {
      ProtoType::Message(ProtoPath {
        name: "TimeOfDay".into(),
        package: "google.type".into(),
        file: "google/type/timeofday.proto".into(),
      })
    }
  }

  impl AsProtoType for DateTime {
    fn proto_type() -> ProtoType {
      ProtoType::Message(ProtoPath {
        name: "DateTime".into(),
        package: "google.type".into(),
        file: "google/type/datetime.proto".into(),
      })
    }
  }
}

#[cfg(feature = "common-types")]
mod rpc_types {
  use super::*;
  use proto_types::*;

  macro_rules! impl_types {
    ($($name:ident => $file:literal),*) => {
      $(
        impl AsProtoType for $name {
          fn proto_type() -> ProtoType {
            ProtoType::Message(ProtoPath {
              name: stringify!($name).into(),
              package: "google.rpc".into(),
              file: concat!("google/rpc/", $file).into(),
            })
          }
        }
      )*
    };
  }

  impl_types!(
    ErrorInfo => "error_details",
    DebugInfo => "error_details",
    RetryInfo => "error_details",
    QuotaFailure => "error_details",
    PreconditionFailure => "error_details",
    BadRequest => "error_details",
    RequestInfo => "error_details",
    ResourceInfo => "error_details",
    Help => "error_details",
    LocalizedMessage => "error_details",
    HttpRequest => "http",
    HttpResponse => "http",
    HttpHeader => "http"
  );

  impl AsProtoType for quota_failure::Violation {
    fn proto_type() -> ProtoType {
      ProtoType::Message(ProtoPath {
        name: "QuotaFailure.Violation".into(),
        package: "google.rpc".into(),
        file: "google/rpc/error_details.proto".into(),
      })
    }
  }

  impl AsProtoType for precondition_failure::Violation {
    fn proto_type() -> ProtoType {
      ProtoType::Message(ProtoPath {
        name: "PreconditionFailure.Violation".into(),
        package: "google.rpc".into(),
        file: "google/rpc/error_details.proto".into(),
      })
    }
  }

  impl AsProtoType for bad_request::FieldViolation {
    fn proto_type() -> ProtoType {
      ProtoType::Message(ProtoPath {
        name: "BadRequest.FieldViolation".into(),
        package: "google.rpc".into(),
        file: "google/rpc/error_details.proto".into(),
      })
    }
  }

  impl AsProtoType for help::Link {
    fn proto_type() -> ProtoType {
      ProtoType::Message(ProtoPath {
        name: "Help.Link".into(),
        package: "google.rpc".into(),
        file: "google/rpc/error_details.proto".into(),
      })
    }
  }
}
