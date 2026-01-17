use crate::*;

use proto_types::protovalidate::*;

// Here we use a generic for the target of the validator
// AND an assoc. type for the actual type being validated
// so that it can be proxied by wrappers (like with Sint32, Fixed32, enums, etc...).
// Same for `ValidatorBuilderFor`.
pub trait Validator<T: ?Sized>: Into<ProtoOption> {
  type Target: ToOwned + ?Sized;
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
  fn check_cel_programs_with(
    &self,
    _val: <Self::Target as ToOwned>::Owned,
  ) -> Result<(), Vec<CelError>>;

  #[cfg(feature = "cel")]
  fn check_cel_programs(&self) -> Result<(), Vec<CelError>>;

  fn validate<V>(&self, ctx: &mut ValidationCtx, val: Option<&V>) -> bool
  where
    V: Borrow<Self::Target> + ?Sized;
}

pub trait ValidatorBuilderFor<T: ?Sized>: Default {
  type Target: ?Sized;
  type Validator: Validator<T, Target = Self::Target>;

  fn build_validator(self) -> Self::Validator;
}

pub trait ProtoValidator {
  type Target: ?Sized;
  type Validator: Validator<Self, Target = Self::Target> + Clone;
  type Builder: ValidatorBuilderFor<Self, Validator = Self::Validator>;

  #[must_use]
  #[inline]
  fn default_validator() -> Option<Self::Validator> {
    None
  }

  #[inline]
  #[must_use]
  fn validator_builder() -> Self::Builder {
    Self::Builder::default()
  }

  fn validator_from_closure<F, FinalBuilder>(config_fn: F) -> Self::Validator
  where
    F: FnOnce(Self::Builder) -> FinalBuilder,
    FinalBuilder: ValidatorBuilderFor<Self, Validator = Self::Validator>,
  {
    let initial_builder = Self::validator_builder();

    config_fn(initial_builder).build_validator()
  }
}

pub trait ValidateWith: Sized {
  type Item: ProtoValidator;

  fn validate_with<V, S>(&self, validator: &V) -> Result<(), Violations>
  where
    V: Validator<S, Target = <Self::Item as ProtoValidator>::Target>;

  fn validate_with_closure<F, FinalBuilder>(&self, config: F) -> Result<(), Violations>
  where
    F: FnOnce(<Self::Item as ProtoValidator>::Builder) -> FinalBuilder,
    FinalBuilder:
      ValidatorBuilderFor<Self::Item, Validator = <Self::Item as ProtoValidator>::Validator>,
  {
    let builder = <Self::Item as ProtoValidator>::validator_builder();
    let validator = config(builder).build_validator();
    self.validate_with(&validator)
  }
}

impl<T> ValidateWith for Option<T>
where
  T: ValidateWith + Borrow<<T::Item as ProtoValidator>::Target>,
{
  type Item = T::Item;

  fn validate_with<V, S>(&self, validator: &V) -> Result<(), Violations>
  where
    V: Validator<S, Target = <T::Item as ProtoValidator>::Target>,
  {
    let mut ctx = ValidationCtx::default();

    let val_ref = self.as_ref().map(|v| v.borrow());

    validator.validate(&mut ctx, val_ref);

    if ctx.violations.is_empty() {
      Ok(())
    } else {
      Err(ctx.violations.into_violations())
    }
  }
}

impl ValidateWith for &str {
  type Item = String;

  fn validate_with<V, S>(&self, validator: &V) -> Result<(), Violations>
  where
    V: Validator<S, Target = <Self::Item as ProtoValidator>::Target>,
  {
    let mut ctx = ValidationCtx::default();

    validator.validate(&mut ctx, Some(self));

    if ctx.violations.is_empty() {
      Ok(())
    } else {
      Err(ctx.violations.into_violations())
    }
  }
}

impl<T> ValidateWith for T
where
  T: ProtoValidator + Borrow<T::Target>,
{
  type Item = Self;

  fn validate_with<V, S>(&self, validator: &V) -> Result<(), Violations>
  where
    V: Validator<S, Target = T::Target>,
  {
    let mut ctx = ValidationCtx::default();

    validator.validate(&mut ctx, Some(self.borrow()));

    if ctx.violations.is_empty() {
      Ok(())
    } else {
      Err(ctx.violations.into_violations())
    }
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
