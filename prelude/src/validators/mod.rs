use crate::*;
mod common_strings;
use std::{fmt::Debug, hash::Hash, sync::Arc};

use common_strings::*;
use proto_types::protovalidate::*;
use protocheck_core::{
  ordered_float::OrderedFloat,
  validators::{containing::ListRules, well_known_strings::*},
};

// Here we use a generic for the target of the validator
// AND an assoc. type for the actual type being validated
// so that it can be proxied by wrappers (like with Sint32, Fixed32, enums, etc...).
// Same for `ValidatorBuilderFor`.
pub trait Validator<T>: Into<ProtoOption> {
  type Target: Default;
  type UniqueStore<'a>: UniqueStore<'a, Item = Self::Target>
  where
    Self: 'a;

  fn make_unique_store<'a>(&self, cap: usize) -> Self::UniqueStore<'a>;

  // This one cannot be testing only because it is used in the schema impl below
  fn cel_rules(&self) -> Vec<&'static CelRule> {
    self
      .cel_programs()
      .into_iter()
      .map(|p| &p.rule)
      .collect()
  }

  fn cel_programs(&self) -> Vec<&'static CelProgram> {
    vec![]
  }

  fn into_schema(self) -> FieldValidator {
    FieldValidator {
      cel_rules: self.cel_rules(),
      schema: self.into(),
    }
  }

  #[cfg(feature = "testing")]
  fn check_consistency(&self) -> Result<(), Vec<String>> {
    Ok(())
  }

  #[cfg(feature = "testing")]
  fn check_cel_programs_with(&self, _val: Self::Target) -> Result<(), Vec<CelError>>;

  #[cfg(feature = "testing")]
  fn check_cel_programs(&self) -> Result<(), Vec<CelError>> {
    self.check_cel_programs_with(Self::Target::default())
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

pub type CachedList<T> = LazyLock<[T]>;

type OptionValueList = Vec<(Arc<str>, OptionValue)>;

impl From<Ignore> for OptionValue {
  fn from(value: Ignore) -> Self {
    let name = match value {
      Ignore::Unspecified => IGNORE_UNSPECIFIED.clone(),
      Ignore::IfZeroValue => IGNORE_IF_ZERO_VALUE.clone(),
      Ignore::Always => IGNORE_ALWAYS.clone(),
    };

    Self::Enum(name)
  }
}

macro_rules! impl_cel_method {
  ($builder:ident) => {
    paste::paste! {
      impl < S: [< $builder:snake >]::State> $builder< S>
      {
        /// Adds a custom CEL rule to this validator.
        /// Use the [`cel_program`] or [`inline_cel_program`] macros to build a static program.
        pub fn cel(mut self, program: &'static CelProgram) -> Self {
          self.cel.push(program);
          self
        }
      }
    }
  };
}

macro_rules! impl_ignore {
  ($builder:ident) => {
    paste::paste! {
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

pub(crate) fn format_list<T: Display, I: IntoIterator<Item = T>>(list: I) -> String {
  let mut string = String::new();
  let mut iter = list.into_iter().peekable();

  while let Some(item) = iter.next() {
    write!(string, "{item}").unwrap();

    if iter.peek().is_some() {
      string.push_str(", ");
    }
  }

  string
}

#[cfg(feature = "testing")]
pub(crate) fn check_list_rules<T>(
  in_list: Option<&'static SortedList<T>>,
  not_in_list: Option<&'static SortedList<T>>,
) -> Result<(), OverlappingListsError<T>>
where
  T: Debug + PartialEq + Eq + Hash + Ord + Clone,
{
  if let Some(in_list) = in_list
    && let Some(not_in_list) = not_in_list
  {
    let mut overlapping: Vec<T> = Vec::with_capacity(in_list.len());

    for item in in_list {
      let is_overlapping = not_in_list.contains(item);

      if is_overlapping {
        overlapping.push(item.clone());
      }
    }

    if overlapping.is_empty() {
      return Ok(());
    } else {
      return Err(OverlappingListsError { overlapping });
    }
  }

  Ok(())
}

pub(crate) struct OverlappingListsError<T: Debug> {
  pub overlapping: Vec<T>,
}

impl<T: Debug> Display for OverlappingListsError<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "The following values are both allowed and forbidden:")?;

    for item in &self.overlapping {
      let _ = writeln!(f, "  - {item:#?}");
    }

    Ok(())
  }
}

#[cfg(feature = "testing")]
pub(crate) fn check_comparable_rules<T>(
  lt: Option<T>,
  lte: Option<T>,
  gt: Option<T>,
  gte: Option<T>,
) -> Result<(), &'static str>
where
  T: Display + PartialEq + PartialOrd + Copy,
{
  if lt.is_some() && lte.is_some() {
    return Err("Lt and Lte cannot be used together.");
  }

  if gt.is_some() && gte.is_some() {
    return Err("Gt and Gte cannot be used together.");
  }

  if let Some(lt) = lt {
    if let Some(gt) = gt
      && lt <= gt
    {
      return Err("Lt cannot be smaller than or equal to Gt");
    }

    if let Some(gte) = gte
      && lt <= gte
    {
      return Err("Lte cannot be smaller than or equal to Gte");
    }
  }

  if let Some(lte) = lte {
    if let Some(gt) = gt
      && lte <= gt
    {
      return Err("Lte cannot be smaller than or equal to Gt");
    }

    if let Some(gte) = gte
      && lte < gte
    {
      return Err("Lte cannot be smaller than Gte");
    }
  }

  Ok(())
}
