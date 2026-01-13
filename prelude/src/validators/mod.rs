use crate::*;
use std::{fmt::Debug, hash::Hash, sync::Arc};

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

  fn into_schema(self) -> FieldValidatorSchema {
    FieldValidatorSchema {
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

pub(crate) trait IsDefault: Default + PartialEq {
  fn is_default(&self) -> bool {
    (*self) == Self::default()
  }
}

impl<T: Default + PartialEq> IsDefault for T {}

pub mod any;
pub mod bool;
mod builder_internals;
pub mod bytes;
mod cel;
pub mod duration;
pub mod enums;
pub mod field_context;
pub mod map;
pub mod message;
pub mod repeated;
pub mod string;
pub mod timestamp;

pub mod floats;
pub use floats::*;
pub mod integers;
pub use integers::*;
pub mod field_mask;
pub use field_mask::*;
pub mod lookup;
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
pub use repeated::*;
pub use string::*;
pub use timestamp::*;
