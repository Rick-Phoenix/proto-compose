macro_rules! length_rule_value {
  ($name:literal, $value:expr) => {
    &LengthRuleValue {
      name: $name,
      value: $value,
    }
  };
}

#[macro_export]
macro_rules! use_proto_file {
  ($file:path) => {
    #[doc(hidden)]
    #[allow(unused)]
    const __PROTO_FILE: $crate::RegistryPath = $file;
  };
}

macro_rules! handle_ignore_always {
  ($ignore:expr) => {
    if matches!($ignore, Ignore::Always) {
      return;
    }
  };
}

macro_rules! handle_ignore_if_zero_value {
  ($ignore:expr, $condition:expr) => {
    if matches!($ignore, Ignore::IfZeroValue) && $condition {
      return;
    }
  };
}

macro_rules! impl_testing_methods {
  () => {
    #[cfg(feature = "cel")]
    fn check_cel_programs_with(&self, val: Self::Target) -> Result<(), Vec<CelError>> {
      if !self.cel.is_empty() {
        test_programs(&self.cel, val)
      } else {
        Ok(())
      }
    }

    fn cel_rules(&self) -> Vec<CelRule> {
      self.cel.iter().map(|p| p.rule.clone()).collect()
    }
  };
}

#[macro_export]
macro_rules! cel_program {
  (id = $id:expr, msg = $msg:expr, expr = $expr:expr) => {
    ::prelude::CelProgram::new($crate::CelRule {
      id: $id.into(),
      message: $msg.into(),
      expression: $expr.into(),
    })
  };

  ($rule:expr) => {
    ::prelude::CelProgram::new($rule)
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
