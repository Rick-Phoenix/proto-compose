#[macro_export]
macro_rules! proto_option {
  ( $name:expr => { $($key:expr => $val:expr),* $(,)? } ) => {
    $crate::ProtoOption {
      name: $name.into(),
      value: $crate::OptionValue::Message($crate::option_message!($($key => $val),*)),
    }
  };

  ($name:expr => $val:expr) => {
    $crate::ProtoOption {
      name: $name.into(),
      value: $val.into(),
    }
  };
}

#[macro_export]
macro_rules! option_list {
  ($list:expr) => {
    $crate::OptionValue::new_list($list)
  };
}

#[macro_export]
macro_rules! option_message {
  ($($key:expr => $val:expr),* $(,)?) => {
    {
      let mut builder = $crate::OptionMessageBuilder::new();
      $(
        builder.set($key, $val);
      )*
      builder.build()
    }
  };

  ($msg:expr) => {
    $crate::OptionValue::new_message($msg)
  }
}

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
    const __PROTO_FILE: $crate::FileReference = $file;
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

    #[doc(hidden)]
    fn cel_rules(&self) -> Vec<CelRule> {
      self.cel.iter().map(|p| p.rule.clone()).collect()
    }
  };
}

#[macro_export]
macro_rules! cel_program {
  (id = $id:expr, msg = $msg:expr, expr = $expr:expr) => {
    $crate::CelRule {
      id: $id.into(),
      message: $msg.into(),
      expression: $expr.into(),
    }
    .into()
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
      }

      impl<S: State> ValidatorBuilderFor<$rust_type> for [< $validator Builder >]<S> {
        type Target = $rust_type;
        type Validator = $validator;

        #[inline]
        fn build_validator(self) -> $validator {
          self.build()
        }
      }
    }
  };
}

macro_rules! impl_proto_type {
  ($rust_type:ty, $proto_type:ident) => {
    impl AsProtoType for $rust_type {
      fn proto_type() -> ProtoType {
        ProtoType::Scalar(ProtoScalar::$proto_type)
      }
    }
  };
}
