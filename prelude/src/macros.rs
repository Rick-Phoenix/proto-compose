#[macro_export]
macro_rules! cel_rule {
  (id = $id:expr, msg = $msg:expr, expr = $expr:expr) => {
    $crate::validators::CelRule {
      id: $id.into(),
      message: $msg.into(),
      expression: $expr.into(),
    }
  };
}

macro_rules! reusable_string {
  ($name:ident) => {
    $crate::paste! {
      pub(crate) static $name: std::sync::LazyLock<std::sync::Arc<str>> =
      std::sync::LazyLock::new(|| stringify!([< $name:lower >]).into());
    }
  };

  ($name:ident, $string:literal) => {
    pub(crate) static $name: std::sync::LazyLock<std::sync::Arc<str>> =
      std::sync::LazyLock::new(|| $string.into());
  };
}

macro_rules! impl_into_option {
  ($validator:ident) => {
    $crate::paste! {
      impl<S: [< $validator:snake _builder >]::IsComplete> From<[< $validator Builder >]<S>> for ProtoOption {
        fn from(value: [< $validator Builder >]<S>) -> ProtoOption {
          let validator = value.build();

          validator.into()
        }
      }
    }
  };
}

macro_rules! impl_validator {
  ($validator:ident, $rust_type:ty) => {
    $crate::paste! {
      impl ProtoValidator<$rust_type> for ValidatorMap {
        type Builder = [< $validator Builder >];

        fn builder() -> Self::Builder {
          $validator::builder()
        }
      }

      impl<S: State> ValidatorBuilderFor<$rust_type> for [< $validator Builder >]<S> {}
    }
  };
}

macro_rules! impl_proto_type {
  ($rust_type:ty, $proto_type:expr) => {
    impl AsProtoType for $rust_type {
      fn proto_type() -> ProtoType {
        ProtoType::Single(TypeInfo {
          name: $proto_type,
          path: None,
        })
      }
    }
  };
}
