use crate::*;
mod common_strings;
use std::{fmt::Debug, hash::Hash, sync::Arc};

use common_strings::*;
use proto_types::protovalidate::*;

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

  fn cel_rules(&self) -> Vec<CelRule>;

  fn into_schema(self) -> FieldValidator {
    FieldValidator {
      cel_rules: self.cel_rules(),
      schema: self.into(),
    }
  }

  fn check_consistency(&self) -> Result<(), Vec<ConsistencyError>>;

  #[cfg(feature = "cel")]
  fn check_cel_programs_with(&self, _val: Self::Target) -> Result<(), Vec<CelError>>;

  #[cfg(feature = "cel")]
  fn check_cel_programs(&self) -> Result<(), Vec<CelError>> {
    self.check_cel_programs_with(Self::Target::default())
  }

  fn validate(&self, ctx: &mut ValidationCtx, val: Option<&Self::Target>);
}

pub trait ValidatorBuilderFor<T>: Default {
  type Target;
  type Validator: Validator<T, Target = Self::Target>;

  fn build_validator(self) -> Self::Validator;
}

pub trait ProtoValidator: std::marker::Sized {
  type Target;
  type Validator: Validator<Self, Target = Self::Target> + Clone;
  type Builder: ValidatorBuilderFor<Self, Validator = Self::Validator>;

  #[doc(hidden)]
  #[must_use]
  #[inline]
  fn default_validator() -> Option<Self::Validator> {
    None
  }

  #[doc(hidden)]
  #[inline]
  #[must_use]
  fn validator_builder() -> Self::Builder {
    Self::Builder::default()
  }

  #[doc(hidden)]
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

    Self::Enum(name)
  }
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
mod oneof;
mod repeated;
mod string;
mod timestamp;

mod floats;
pub use floats::*;
mod integers;
pub use integers::*;
mod field_mask;
pub use field_mask::*;
mod lookup;
pub use lookup::*;

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
pub use oneof::*;
pub use repeated::*;
pub use string::*;
pub use timestamp::*;

pub(crate) fn check_list_rules<T>(
  in_list: Option<&StaticLookup<T>>,
  not_in_list: Option<&StaticLookup<T>>,
) -> Result<(), OverlappingListsError>
where
  T: Debug + PartialEq + Eq + Hash + Ord + Clone + ListFormatter,
{
  if let Some(in_list) = in_list
    && let Some(not_in_list) = not_in_list
  {
    let mut overlapping: Vec<T> = Vec::with_capacity(in_list.items.len());

    for item in &in_list.items {
      let is_overlapping = not_in_list.items.contains(item);

      if is_overlapping {
        overlapping.push(item.clone());
      }
    }

    if overlapping.is_empty() {
      return Ok(());
    } else {
      return Err(OverlappingListsError {
        overlapping: overlapping
          .into_iter()
          .map(|i| format!("{i:#?}"))
          .collect(),
      });
    }
  }

  Ok(())
}

#[derive(Debug)]
pub struct OverlappingListsError {
  pub overlapping: Vec<String>,
}

impl core::error::Error for OverlappingListsError {}

impl Display for OverlappingListsError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "The following values are both allowed and forbidden:")?;

    for item in &self.overlapping {
      let _ = writeln!(f, "  - {item}");
    }

    Ok(())
  }
}

#[allow(clippy::useless_let_if_seq)]
pub(crate) fn check_comparable_rules<T>(
  lt: Option<T>,
  lte: Option<T>,
  gt: Option<T>,
  gte: Option<T>,
) -> Result<(), ConsistencyError>
where
  T: Display + PartialEq + PartialOrd + Copy,
{
  let mut err: Option<&str> = None;

  if lt.is_some() && lte.is_some() {
    err = Some("Lt and Lte cannot be used together.");
  }

  if gt.is_some() && gte.is_some() {
    err = Some("Gt and Gte cannot be used together.");
  }

  if let Some(lt) = lt {
    if let Some(gt) = gt
      && lt <= gt
    {
      err = Some("Lt cannot be smaller than or equal to Gt");
    }

    if let Some(gte) = gte
      && lt <= gte
    {
      err = Some("Lte cannot be smaller than or equal to Gte");
    }
  }

  if let Some(lte) = lte {
    if let Some(gt) = gt
      && lte <= gt
    {
      err = Some("Lte cannot be smaller than or equal to Gt");
    }

    if let Some(gte) = gte
      && lte < gte
    {
      err = Some("Lte cannot be smaller than Gte");
    }
  }

  if let Some(err) = err {
    Err(ConsistencyError::ContradictoryInput(err.to_string()))
  } else {
    Ok(())
  }
}

#[cfg(feature = "cel")]
pub trait IntoCel: Into<::cel::Value> {}
#[cfg(feature = "cel")]
impl<T: Into<::cel::Value>> IntoCel for T {}

#[cfg(not(feature = "cel"))]
pub trait IntoCel {}
#[cfg(not(feature = "cel"))]
impl<T> IntoCel for T {}
