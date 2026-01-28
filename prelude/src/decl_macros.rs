// For docs
#[allow(unused)]
use crate::*;

macro_rules! pluralize {
  ($count:expr) => {
    if $count != 1 { "s" } else { "" }
  };
}

macro_rules! custom_error_messages_method {
  ($kind:ident) => {
    paste! {
      #[inline]
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

#[doc(hidden)]
#[cfg(feature = "inventory")]
#[macro_export]
macro_rules! register_proto_data {
  ($($tokens:tt)*) => {
    $crate::inventory::submit! { $($tokens)* }
  };
}

#[doc(hidden)]
#[cfg(not(feature = "inventory"))]
#[macro_export]
macro_rules! register_proto_data {
  ($($tokens:tt)*) => {};
}

/// This macro can be used to manually create a package schema if the inventory feature is not available.
///
/// The first argument is the name of the package, and the second argument is a collection of the files to include.
///
/// ```
/// use prelude::*;
///
/// let manual_file = file_schema!(
///   name = "test.proto",
/// );
///
/// let manual_pkg = package_schema!("my_pkg", files = [ manual_file ]);
/// ```
#[macro_export]
macro_rules! package_schema {
  ($name:expr, files = $files:expr) => {
    $crate::Package::new($name).with_files($files)
  };
}

/// This macro can be used to generate a [`ProtoOption`] with a concise syntax.
///
/// The input can be a single `key => value`, where the key should support [`Into`] [`FixedStr`]  and the value should support [`Into`] [`OptionValue`], or a bracketed series or key-value pairs to generate an [`OptionValue::Message`] for the value.
///
/// # Examples
///
/// ```
/// use prelude::*;
///
/// let option = proto_option!("is_cool" => true);
/// assert_eq!(option, ProtoOption { name: "is_cool".into(), value: true.into() });
///
/// let object_like_option = proto_option!("is_cool" => { "answer" => true });
/// assert_eq!(object_like_option.name, "is_cool");
/// assert_eq!(object_like_option.value, OptionValue::Message(option_message!("answer" => true)));
/// ```
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

/// This macro can be used to create an [`OptionValue::List`] from an iterator of items that implement [`Into`] [`OptionValue`].
///
/// # Examples
///
/// ```
/// use prelude::*;
///
/// let list = option_list!([ 1, 2 ]);
/// assert!(matches!(list, OptionValue::List(_)));
/// ```
#[macro_export]
macro_rules! option_list {
  ($list:expr) => {
    $crate::OptionValue::new_list($list)
  };
}

/// This macro can be used to create an object-like protobuf option. It follows the syntax of crates like maplit, for creating key-value pairs.
///
/// # Examples
///
/// ```
/// use prelude::*;
///
/// let option = option_message!("is_cool" => true);
/// let value = option.get("is_cool").unwrap();
///
/// assert_eq!(value, &OptionValue::Bool(true));
/// ```
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

/// Brings a pre-defined proto file handle in scope so that it can be picked up by the proto items defined in the module where it's called.
///
/// It **retains** the `extern_path` of the original file, so it should be used only if the items are meant to be re-exported by the module where the original file is defined.
///
/// # Examples
///
/// ```rust
/// use prelude::*;
/// mod example {
///   use super::*;
///
///   proto_package!(MY_PKG, name = "my_pkg");
///   define_proto_file!(MY_FILE, name = "my_file.proto", package = MY_PKG);
///
///   pub use re_exported::Msg;
///
///   pub fn mod_path() -> &'static str {
///     module_path!()
///   }
///  
///   mod re_exported {
///     use super::MY_FILE;
///     use prelude::*;
///  
///     // The file is now in scope, and will be picked up automatically by all items defined in this module
///     // The items will have the extern path of the parent, so `::cratename::example`
///     inherit_proto_file!(MY_FILE);
///
///     // This will have the extern path `::cratename::example::Msg`
///     #[proto_message]
///     pub struct Msg {
///       pub id: i32
///     }
///   }
/// }
///
/// fn main() {
///   assert_eq!(example::Msg::proto_schema().rust_path, &format!("::{}::Msg", example::mod_path()));
/// }
/// ```
#[macro_export]
macro_rules! inherit_proto_file {
  ($file:path) => {
    #[doc(hidden)]
    #[allow(unused)]
    const __PROTO_FILE: $crate::FileReference = $file;
  };
}

/// Brings a pre-defined proto file handle in scope so that it can be picked up by the proto items defined in the module where it's called.
///
/// The items defined in the module where this is called will have the `extern_path` set to the output of `module_path!()`. For re-exported items that are meant to inherit the same path as the parent module, use the [`inherit_proto_file`] macro instead.
///
/// # Examples
///
/// ```rust
/// use prelude::*;
///
/// mod example {
///   use super::*;
///   proto_package!(MY_PKG, name = "my_pkg");
///   define_proto_file!(MY_FILE, name = "my_file.proto", package = MY_PKG);
///  
///   pub mod submod {
///     use super::MY_FILE;
///     use prelude::*;
///
///     pub fn mod_path() -> &'static str {
///       module_path!()
///     }
///  
///     // The file is now in scope, and will be picked up automatically by all items defined in this module
///     // The items will have the extern path of the `module_path!()` output in here, so `::cratename::example::submod`
///     use_proto_file!(MY_FILE);
///
///     // This will have the extern path `::cratename::example::submod::Msg`
///     #[proto_message]
///     pub struct Msg {
///       pub id: i32
///     }
///   }
/// }
///
/// fn main() {
///   assert_eq!(example::submod::Msg::proto_schema().rust_path, &format!("::{}::Msg", example::submod::mod_path()));
/// }
/// ```
#[macro_export]
macro_rules! use_proto_file {
  ($file:path) => {
    #[doc(hidden)]
    #[allow(unused)]
    const __PROTO_FILE: $crate::FileReference = ::prelude::FileReference {
      name: $file.name,
      package: $file.package,
      extern_path: ::core::module_path!(),
    };
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
    #[inline(never)]
    #[cold]
    fn cel_rules(&self) -> Vec<CelRule> {
      self
        .cel
        .iter()
        .map(|p| p.rule().clone())
        .collect()
    }
  };
}

/// Defines a new [`CelProgram`].
///
/// The inputs, in positional order, are:
/// - id (expr, Into<[`FixedStr`]>): The id of the specific CEL rule. It should be unique within the same message scope.
/// - msg (expr, Into<[`FixedStr`]>): The error message associated with the given rule.
/// - expr (expr, Into<[`FixedStr`]>): The actual CEL expression to use when validating the target.
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
