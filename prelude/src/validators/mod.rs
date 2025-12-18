use crate::*;
mod common_strings;
use std::{fmt::Debug, hash::Hash, sync::Arc};

use common_strings::*;
use proto_types::{field_descriptor_proto::Type, protovalidate::*};
use protocheck_core::{
  ordered_float::OrderedFloat,
  validators::{
    containing::{ItemLookup, ListRules},
    repeated::UniqueItem,
    well_known_strings::*,
  },
};

pub trait Validator<T>: Into<ProtoOption> {
  type Target: Default;

  #[cfg(feature = "testing")]
  fn cel_rules(&self) -> Vec<&'static CelRule> {
    Vec::new()
  }

  #[cfg(feature = "testing")]
  fn check_cel_programs_with(&self, _val: Self::Target) -> Result<(), Vec<CelError>> {
    Ok(())
  }

  #[cfg(feature = "testing")]
  fn check_cel_programs(&self) -> Result<(), Vec<CelError>> {
    self.check_cel_programs_with(Self::Target::default())
  }

  fn into_schema(self) -> FieldValidator {
    FieldValidator {
      cel_rules: self.cel_rules(),
      schema: self.into(),
    }
  }

  fn validate(
    &self,
    field_context: &FieldContext,
    parent_elements: &mut Vec<FieldPathElement>,
    val: Option<&Self::Target>,
  ) -> Result<(), Violations>;
}

pub trait ValidatorBuilderFor<T> {
  type Target;
  type Validator: Validator<T, Target = Self::Target>;

  fn build_validator(self) -> Self::Validator;
}

pub trait ProtoValidator: std::marker::Sized {
  type Target;
  type Validator: Validator<Self, Target = Self::Target>;
  type Builder: ValidatorBuilderFor<Self, Validator = Self::Validator>;

  fn validator_builder() -> Self::Builder;

  fn validator_from_closure<F, FinalBuilder>(config_fn: F) -> Self::Validator
  where
    F: FnOnce(Self::Builder) -> FinalBuilder,
    FinalBuilder: ValidatorBuilderFor<Self, Validator = Self::Validator>,
  {
    let initial_builder = Self::validator_builder();

    config_fn(initial_builder).build_validator()
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
      if !$validator.cel.is_empty() {
        let rule_values: Vec<OptionValue> = $validator
          .cel
          .iter()
          .map(|program| program.rule.clone().into())
          .collect();
        $values.push((CEL.clone(), OptionValue::List(rule_values.into())));
      }
    };
  }

  macro_rules! insert_list_option {
    (
    $validator:ident,
    $values:ident,
    $field:ident
  ) => {
      $crate::paste! {
        if let Some(value) = $validator.$field {
          $values.push(([< $field:snake:upper >].clone(), OptionValue::new_list(value)))
        }
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

  macro_rules! insert_boolean_option {
    (
    $validator:ident,
    $values:ident,
    $field:ident
  ) => {
      $crate::paste! {
        if $validator.$field {
          $values.push(([< $field:snake:upper >].clone(), OptionValue::Bool($validator.$field)));
        }
      }
    };
  }
}

pub(crate) trait IsDefault: Default + PartialEq {
  fn is_default(&self) -> bool {
    (*self) == Self::default()
  }
}

impl<T: Default + PartialEq> IsDefault for T {}

mod any;
mod bool;
mod builder_internals;
mod bytes;
mod cel;
mod duration;
mod enums;
pub mod field_context;
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
pub use field_context::*;
pub use map::*;
pub use message::*;
pub use numeric::*;
pub use oneof::*;
pub use repeated::*;
pub use string::*;
pub use timestamp::*;
