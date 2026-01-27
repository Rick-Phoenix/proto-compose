use proto_types::{Any, Code, Duration, Empty, FieldMask, Status, Timestamp};

use crate::*;

macro_rules! impl_no_op_validator {
  ($($name:ty),*) => {
    $(
      impl ProtoValidation for $name {
        #[doc(hidden)]
        type Builder = NoOpValidatorBuilder<Self>;
        #[doc(hidden)]
        type Stored = Self;
        #[doc(hidden)]
        type Target = Self;
        #[doc(hidden)]
        type Validator = NoOpValidator<Self>;

        type UniqueStore<'a>
          = LinearRefStore<'a, Self>
        where
          Self: 'a;

        #[doc(hidden)]
        const HAS_DEFAULT_VALIDATOR: bool = false;
      }

      impl ValidatedMessage for $name {
        #[inline(always)]
        #[doc(hidden)]
        fn validate_with_ctx(&self, _: &mut ValidationCtx) -> ValidationResult {
          Ok(IsValid::Yes)
        }
      }
    )*
  };
}

impl_no_op_validator!(Empty, Status, Code, ());

#[derive(Clone, Copy, Default)]
pub struct NoOpValidator<T: ?Sized>(PhantomData<T>);

impl<T: ?Sized + Send + Sync + ToOwned> Validator<T> for NoOpValidator<T> {
  type Target = T;

  #[inline(always)]
  fn validate_core<V>(&self, _ctx: &mut ValidationCtx, _val: Option<&V>) -> ValidationResult
  where
    V: Borrow<Self::Target> + ?Sized,
  {
    Ok(IsValid::Yes)
  }
}

pub struct NoOpValidatorBuilder<T: ?Sized>(PhantomData<T>);

impl<T: ?Sized> Default for NoOpValidatorBuilder<T> {
  fn default() -> Self {
    Self(Default::default())
  }
}

impl<T> ValidatorBuilderFor<T> for NoOpValidatorBuilder<T>
where
  T: ?Sized + Send + Sync + ToOwned,
{
  type Target = T;
  type Validator = NoOpValidator<T>;
  fn build_validator(self) -> Self::Validator {
    NoOpValidator(PhantomData)
  }
}

macro_rules! impl_msg_path {
  ($name:ident, $package:expr, $file:expr) => {
    impl MessagePath for $name {
      fn proto_path() -> ProtoPath {
        ProtoPath {
          name: stringify!($name).into(),
          package: $package.into(),
          file: $file.into(),
        }
      }
    }

    impl AsProtoType for $name {
      fn proto_type() -> ProtoType {
        ProtoType::Message(Self::proto_path())
      }
    }
  };
}

macro_rules! impl_protobuf_types {
  ($($name:ident),*) => {
    paste! {
      $(
        impl_msg_path!(
          $name,
          "google.protobuf",
          concat!("google/protobuf/", stringify!([< $name:snake >]), ".proto")
        );
      )*
    }
  };
}

impl_protobuf_types!(Duration, Timestamp, FieldMask, Empty, Any);

impl MessagePath for () {
  fn proto_path() -> ProtoPath {
    ProtoPath {
      name: "Empty".into(),
      package: "google.protobuf".into(),
      file: "google/protobuf/empty.proto".into(),
    }
  }
}

impl AsProtoType for () {
  fn proto_type() -> ProtoType {
    ProtoType::Message(Self::proto_path())
  }
}

impl_msg_path!(Status, "google.rpc", "google/rpc/status.proto");

impl AsProtoType for Code {
  fn proto_type() -> ProtoType {
    ProtoType::Enum(ProtoPath {
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

  macro_rules! file_name {
    ($name:ident, ) => {
      paste! {
        concat!("google/type/", stringify!([ < $name:snake > ]), ".proto")
      }
    };

    ($name:ident, $manual:literal) => {
      concat!("google/type/", $manual, ".proto")
    };
  }

  macro_rules! impl_types {
    ($($name:ident $(=> $file:literal)?),*) => {
      paste! {
        $(
          impl AsProtoType for $name {
            fn proto_type() -> ProtoType {
              ProtoType::Message(
                Self::proto_path()
              )
            }
          }

          impl MessagePath for $name {
            fn proto_path() -> ProtoPath {
              ProtoPath {
                name: stringify!($name).into(),
                package: "google.type".into(),
                file: file_name!($name, $($file)?).into(),
              }
            }
          }

          impl_no_op_validator!($name);
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
    Month,
    DateTime => "datetime",
    TimeZone => "datetime",
    LatLng => "latlng",
    TimeOfDay => "timeofday"
  );

  impl_no_op_validator!(DayOfWeek);

  impl AsProtoType for DayOfWeek {
    fn proto_type() -> ProtoType {
      ProtoType::Enum(ProtoPath {
        name: "DayOfWeek".into(),
        package: "google.type".into(),
        file: "google/type/dayofweek.proto".into(),
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
            ProtoType::Message(
              Self::proto_path()
            )
          }
        }

        impl MessagePath for $name {
          fn proto_path() -> ProtoPath {
            ProtoPath {
              name: stringify!($name).into(),
              package: "google.rpc".into(),
              file: concat!("google/rpc/", $file, ".proto").into(),
            }
          }
        }

        impl_no_op_validator!($name);
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

  impl_no_op_validator!(
    quota_failure::Violation,
    precondition_failure::Violation,
    bad_request::FieldViolation,
    help::Link
  );

  impl MessagePath for quota_failure::Violation {
    fn proto_path() -> ProtoPath {
      ProtoPath {
        name: "QuotaFailure.Violation".into(),
        package: "google.rpc".into(),
        file: "google/rpc/error_details.proto".into(),
      }
    }
  }

  impl AsProtoType for quota_failure::Violation {
    fn proto_type() -> ProtoType {
      ProtoType::Message(Self::proto_path())
    }
  }

  impl MessagePath for precondition_failure::Violation {
    fn proto_path() -> ProtoPath {
      ProtoPath {
        name: "PreconditionFailure.Violation".into(),
        package: "google.rpc".into(),
        file: "google/rpc/error_details.proto".into(),
      }
    }
  }

  impl AsProtoType for precondition_failure::Violation {
    fn proto_type() -> ProtoType {
      ProtoType::Message(Self::proto_path())
    }
  }

  impl MessagePath for bad_request::FieldViolation {
    fn proto_path() -> ProtoPath {
      ProtoPath {
        name: "BadRequest.FieldViolation".into(),
        package: "google.rpc".into(),
        file: "google/rpc/error_details.proto".into(),
      }
    }
  }

  impl AsProtoType for bad_request::FieldViolation {
    fn proto_type() -> ProtoType {
      ProtoType::Message(Self::proto_path())
    }
  }

  impl MessagePath for help::Link {
    fn proto_path() -> ProtoPath {
      ProtoPath {
        name: "Help.Link".into(),
        package: "google.rpc".into(),
        file: "google/rpc/error_details.proto".into(),
      }
    }
  }

  impl AsProtoType for help::Link {
    fn proto_type() -> ProtoType {
      ProtoType::Message(Self::proto_path())
    }
  }
}
