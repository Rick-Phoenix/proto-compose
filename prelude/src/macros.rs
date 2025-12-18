macro_rules! handle_ignore_always {
  ($ignore:expr) => {
    if matches!($ignore, Some(Ignore::Always)) {
      return Ok(());
    }
  };
}

macro_rules! handle_ignore_if_zero_value {
  ($ignore:expr, $condition:expr) => {
    if matches!($ignore, Some(Ignore::IfZeroValue)) && $condition {
      return Ok(());
    }
  };
}

macro_rules! impl_testing_methods {
  () => {
    #[cfg(feature = "testing")]
    fn check_cel_programs_with(&self, val: Self::Target) -> Result<(), Vec<CelError>> {
      if !self.cel.is_empty() {
        test_programs(&self.cel, val)
      } else {
        Ok(())
      }
    }

    #[cfg(feature = "testing")]
    fn cel_rules(&self) -> Vec<&'static CelRule> {
      self.cel.iter().map(|prog| &prog.rule).collect()
    }
  };
}

#[macro_export]
macro_rules! cel_rule {
  (id = $id:expr, msg = $msg:expr, expr = $expr:expr) => {
    $crate::CelRule {
      id: $id.into(),
      message: $msg.into(),
      expression: $expr.into(),
    }
  };
}

#[macro_export]
macro_rules! cel_program {
  (id = $id:expr, msg = $msg:expr, expr = $expr:expr) => {
    std::sync::LazyLock::new(|| {
      let rule = $crate::CelRule {
        id: $id.into(),
        message: $msg.into(),
        expression: $expr.into(),
      };

      CelProgram::new(rule)
    })
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
      impl ProtoValidator for $rust_type {
        type Target = $rust_type;
        type Validator = $validator;
        type Builder = [< $validator Builder >];

        fn validator_builder() -> Self::Builder {
          $validator::builder()
        }
      }

      impl<S: State> ValidatorBuilderFor<$rust_type> for [< $validator Builder >]<S> {
        type Target = $rust_type;
        type Validator = $validator;

        fn build_validator(self) -> $validator {
          self.build()
        }
      }
    }
  };
}

macro_rules! impl_proto_type {
  ($rust_type:ty, $proto_type:expr) => {
    impl AsProtoType for $rust_type {
      fn proto_type() -> ProtoType {
        ProtoType::Primitive { name: $proto_type }
      }
    }
  };
}
