use crate::*;
mod common_strings;
use std::{fmt::Debug, hash::Hash, sync::Arc};

use common_strings::*;
use proto_types::protovalidate::Ignore;

pub trait Validator: Into<ProtoOption> {
  type Target;

  fn validate(&self, val: &Self::Target) -> Result<(), bool>;
}

pub trait ValidatorBuilderFor<T>: Into<ProtoOption> {
  type Validator: Validator;

  fn build_validator(self) -> Self::Validator;
}

pub trait ProtoValidator<T> {
  type Validator: Validator;
  type Builder: ValidatorBuilderFor<T, Validator = Self::Validator>;

  fn builder() -> Self::Builder;

  fn from_builder<B>(builder: B) -> ProtoOption
  where
    B: ValidatorBuilderFor<T>,
  {
    builder.into()
  }

  fn validator_from_closure<F, FinalBuilder>(config_fn: F) -> Self::Validator
  where
    F: FnOnce(Self::Builder) -> FinalBuilder,
    FinalBuilder: ValidatorBuilderFor<T, Validator = Self::Validator>,
  {
    let initial_builder = Self::builder();

    config_fn(initial_builder).build_validator()
  }

  fn builder_from_closure<F, FinalBuilder>(config_fn: F) -> FinalBuilder
  where
    F: FnOnce(Self::Builder) -> FinalBuilder,
    FinalBuilder: ValidatorBuilderFor<T>,
  {
    let initial_builder = Self::builder();

    config_fn(initial_builder)
  }

  fn build_rules<F, FinalBuilder>(config_fn: F) -> ProtoOption
  where
    F: FnOnce(Self::Builder) -> FinalBuilder,
    FinalBuilder: ValidatorBuilderFor<T>,
  {
    let initial_builder = Self::builder();

    let final_builder = config_fn(initial_builder);

    final_builder.into()
  }
}

type OptionValueList = Vec<(Arc<str>, OptionValue)>;

impl From<Ignore> for OptionValue {
  fn from(value: Ignore) -> Self {
    let name = match value {
      Ignore::Unspecified => IGNORE_UNSPECIFIED.clone(),
      Ignore::IfZeroValue => IGNORE_IF_ZERO_VALUE.clone(),
      Ignore::Always => IGNORE_ALWAYS.clone(),
    };

    OptionValue::Enum(name)
  }
}

fn create_string_list<T: Into<Arc<str>>, I: IntoIterator<Item = T>>(list: I) -> Arc<[Arc<str>]> {
  let new_list: Vec<Arc<str>> = list.into_iter().map(|i| i.into()).collect();

  new_list.into()
}

macro_rules! impl_ignore {
  ($builder:ident) => {
    $crate::paste! {
      impl < S: [< $builder:snake >]::State> $builder< S>
      where
        S::Ignore: [< $builder:snake >]::IsUnset,
      {
        /// Rules defined for this field will be ignored if the field is set to its protobuf zero value.
        pub fn ignore_if_zero_value(self) -> $builder< [< $builder:snake >]::SetIgnore<S>> {
          self.ignore(Ignore::IfZeroValue)
        }

        /// Rules set for this field will always be ignored.
        pub fn ignore_always(self) -> $builder< [< $builder:snake >]::SetIgnore<S>> {
          self.ignore(Ignore::Always)
        }
      }
    }
  };
}

#[macro_use]
mod macros {
  macro_rules! insert_cel_rules {
    ($validator:ident, $values:ident) => {
      if let Some(cel_rules) = $validator.cel {
        let rule_values: Vec<OptionValue> =
          cel_rules.iter().cloned().map(OptionValue::from).collect();
        $values.push((CEL.clone(), OptionValue::List(rule_values.into())));
      }
    };
  }

  macro_rules! insert_option {
    (
    $validator:ident,
    $values:ident,
    $field:ident
  ) => {
      $crate::paste! {
        if let Some(value) = $validator.$field {
          $values.push(([< $field:snake:upper >].clone(), value.into()))
        }
      }
    };
  }
}

mod any;
mod bool;
mod builder_internals;
mod bytes;
mod cel;
mod duration;
mod enums;
mod map;
mod message;
mod numeric;
mod oneof;
mod repeated;
mod string;
mod timestamp;

pub use any::*;
pub use bool::*;
use builder_internals::*;
pub use bytes::*;
pub use cel::*;
pub use duration::*;
pub use enums::*;
pub use map::*;
pub use message::*;
pub use numeric::*;
pub use oneof::*;
pub use repeated::*;
pub use string::*;
pub use timestamp::*;
