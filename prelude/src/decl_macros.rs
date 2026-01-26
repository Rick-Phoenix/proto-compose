macro_rules! pluralize {
  ($count:expr) => {
    if $count != 1 { "s" } else { "" }
  };
}

macro_rules! custom_error_messages_method {
  ($kind:ident) => {
    paste! {
      pub fn with_error_messages(
        mut self,
        error_messages: impl IntoIterator<Item = ([< $kind Violation >], impl Into<FixedStr>)>,
      ) -> [< $kind ValidatorBuilder >]<SetErrorMessages<S>>
    where
      S::ErrorMessages: IsUnset,
      {
        let map: BTreeMap<[< $kind Violation >], FixedStr> = error_messages
        .into_iter()
        .map(|(v, m)| (v, m.into()))
        .collect();

        self.data.error_messages = Some(Box::new(map));

        [< $kind ValidatorBuilder >] {
          _state: PhantomData,
          data: self.data,
        }
      }
    }
  };
}

#[cfg(feature = "inventory")]
#[macro_export]
macro_rules! register_proto_data {
  ($($tokens:tt)*) => {
    $crate::inventory::submit! { $($tokens)* }
  };
}

#[cfg(not(feature = "inventory"))]
#[macro_export]
macro_rules! register_proto_data {
  ($($tokens:tt)*) => {};
}

#[macro_export]
macro_rules! package_schema {
  ($name:expr, files = $files:expr) => {
    $crate::Package::new($name).with_files($files)
  };
}

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
  ($file:path, extern_path = $path:literal) => {
    #[doc(hidden)]
    #[allow(unused)]
    const __PROTO_FILE: $crate::FileReference = ::prelude::FileReference {
      name: $file.name,
      package: $file.package,
      extern_path: $path,
    };
  };

  ($file:path) => {
    #[doc(hidden)]
    #[allow(unused)]
    const __PROTO_FILE: $crate::FileReference = $file;
  };
}

macro_rules! handle_ignore_always {
  ($ignore:expr) => {
    if matches!($ignore, Ignore::Always) {
      return Ok(IsValid::Yes);
    }
  };
}

macro_rules! handle_ignore_if_zero_value {
  ($ignore:expr, $condition:expr) => {
    if matches!($ignore, Ignore::IfZeroValue) && $condition {
      return Ok(IsValid::Yes);
    }
  };
}

macro_rules! impl_testing_methods {
  () => {
    #[cfg(feature = "cel")]
    #[inline(never)]
    #[cold]
    fn check_cel_programs_with(&self, val: Self::Target) -> Result<(), Vec<CelError>> {
      if !self.cel.is_empty() {
        test_programs(&self.cel, val)
      } else {
        Ok(())
      }
    }

    #[cfg(feature = "cel")]
    #[inline(never)]
    #[cold]
    fn check_cel_programs(&self) -> Result<(), Vec<CelError>> {
      self.check_cel_programs_with(Self::Target::default())
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

macro_rules! impl_proto_type {
  ($rust_type:ty, $proto_type:ident) => {
    impl AsProtoType for $rust_type {
      fn proto_type() -> ProtoType {
        ProtoType::Scalar(ProtoScalar::$proto_type)
      }
    }
  };
}

macro_rules! impl_proto_map_key {
  ($rust_type:ty, $enum_ident:ident) => {
    #[doc(hidden)]
    impl AsProtoMapKey for $rust_type {
      #[allow(private_interfaces)]
      const SEALED: crate::proto_type::Sealed = crate::proto_type::Sealed;

      fn as_proto_map_key() -> ProtoMapKey {
        ProtoMapKey::$enum_ident
      }
    }
  };
}
