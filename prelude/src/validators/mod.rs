use crate::*;

use proto_types::protovalidate::*;

pub trait ValidatorResultExt {
  #[allow(private_interfaces)]
  const SEALED: Sealed;

  fn is_valid(&self) -> bool;
  fn is_fail_fast(&self) -> bool;
}

impl ValidatorResultExt for ValidatorResult {
  #[allow(private_interfaces)]
  const SEALED: Sealed = Sealed;

  #[inline]
  fn is_valid(&self) -> bool {
    match self {
      Ok(outcome) => matches!(outcome, IsValid::Yes),
      Err(_) => false,
    }
  }

  #[inline]
  fn is_fail_fast(&self) -> bool {
    self.is_err()
  }
}

#[derive(Debug, Clone, Copy)]
pub struct FailFast;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum IsValid {
  Yes = 1,
  No = 0,
}

impl From<IsValid> for bool {
  fn from(val: IsValid) -> Self {
    match val {
      IsValid::Yes => true,
      IsValid::No => false,
    }
  }
}

impl IsValid {
  #[must_use]
  pub fn is_valid(&self) -> bool {
    (*self).into()
  }

  #[must_use]
  pub const fn merge(self, other: Self) -> Self {
    match (self, other) {
      (Self::Yes, Self::Yes) => Self::Yes,
      _ => Self::No,
    }
  }
}

impl core::ops::BitAndAssign for IsValid {
  fn bitand_assign(&mut self, rhs: Self) {
    *self = self.merge(rhs);
  }
}

pub type ValidatorResult = Result<IsValid, FailFast>;

// Here we use a generic for the target of the validator
// AND an assoc. type for the actual type being validated
// so that it can be proxied by wrappers (like with Sint32, Fixed32, enums, etc...).
// Same for `ValidatorBuilderFor`.
pub trait Validator<T: ?Sized>: Sized {
  type Target: ToOwned + ?Sized;

  #[inline(never)]
  #[cold]
  fn cel_rules(&self) -> Vec<CelRule> {
    vec![]
  }

  #[inline(never)]
  #[cold]
  fn schema(&self) -> Option<ValidatorSchema> {
    None
  }

  #[inline(never)]
  #[cold]
  fn check_consistency(&self) -> Result<(), Vec<ConsistencyError>> {
    Ok(())
  }

  #[cfg(feature = "cel")]
  #[inline(never)]
  #[cold]
  fn check_cel_programs_with(
    &self,
    _val: <Self::Target as ToOwned>::Owned,
  ) -> Result<(), Vec<CelError>> {
    Ok(())
  }

  #[cfg(feature = "cel")]
  #[inline(never)]
  #[cold]
  fn check_cel_programs(&self) -> Result<(), Vec<CelError>> {
    Ok(())
  }

  #[inline]
  fn validate<V>(&self, val: &V) -> Result<(), ViolationsAcc>
  where
    V: Borrow<Self::Target> + ?Sized,
  {
    let mut ctx = ValidationCtx::default();

    let _ = self.validate_core(&mut ctx, Some(val));

    if ctx.violations.is_empty() {
      Ok(())
    } else {
      Err(ctx.violations)
    }
  }

  #[inline]
  fn validate_option<V>(&self, val: Option<&V>) -> Result<(), ViolationsAcc>
  where
    V: Borrow<Self::Target> + ?Sized,
  {
    let mut ctx = ValidationCtx::default();

    let _ = self.validate_core(&mut ctx, val);

    if ctx.violations.is_empty() {
      Ok(())
    } else {
      Err(ctx.violations)
    }
  }

  #[inline]
  fn validate_with_ctx<V>(&self, mut ctx: ValidationCtx, val: &V) -> Result<(), ViolationsAcc>
  where
    V: Borrow<Self::Target> + ?Sized,
  {
    let _ = self.validate_core(&mut ctx, Some(val));

    if ctx.violations.is_empty() {
      Ok(())
    } else {
      Err(ctx.violations)
    }
  }

  #[inline]
  fn validate_option_with_ctx<V>(
    &self,
    mut ctx: ValidationCtx,
    val: Option<&V>,
  ) -> Result<(), ViolationsAcc>
  where
    V: Borrow<Self::Target> + ?Sized,
  {
    let _ = self.validate_core(&mut ctx, val);

    if ctx.violations.is_empty() {
      Ok(())
    } else {
      Err(ctx.violations)
    }
  }

  fn validate_core<V>(&self, ctx: &mut ValidationCtx, val: Option<&V>) -> ValidatorResult
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
  type Stored: Borrow<Self::Target>;
  type Validator: Validator<Self, Target = Self::Target> + Clone + Default;
  type Builder: ValidatorBuilderFor<Self, Validator = Self::Validator>;

  const HAS_DEFAULT_VALIDATOR: bool = false;
  const HAS_SHALLOW_VALIDATION: bool = false;

  type UniqueStore<'a>: UniqueStore<'a, Item = Self::Target>
  where
    Self: 'a;

  #[inline]
  fn make_unique_store<'a>(_validator: &Self::Validator, cap: usize) -> Self::UniqueStore<'a>
  where
    Self: 'a,
  {
    Self::UniqueStore::default_with_capacity(cap)
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

pub(crate) trait IsDefault: Default + PartialEq {
  fn is_default(&self) -> bool {
    (*self) == Self::default()
  }
}

pub struct FnValidator<F, T: ?Sized> {
  func: F,
  _phantom: PhantomData<T>,
}

impl<F, T> Validator<T> for FnValidator<F, T>
where
  T: ToOwned + ?Sized,
  F: Fn(&mut ValidationCtx, Option<&T>) -> ValidatorResult,
{
  type Target = T;

  #[inline]
  fn validate_core<V>(&self, ctx: &mut ValidationCtx, val: Option<&V>) -> ValidatorResult
  where
    V: Borrow<Self::Target> + ?Sized,
  {
    let target = val.map(|v| v.borrow());
    (self.func)(ctx, target)
  }
}

#[inline]
pub const fn from_fn<T, F>(f: F) -> FnValidator<F, T>
where
  T: ?Sized,
  F: Fn(&mut ValidationCtx, Option<&T>) -> ValidatorResult,
{
  FnValidator {
    func: f,
    _phantom: PhantomData,
  }
}

impl<T: Default + PartialEq> IsDefault for T {}
type ErrorMessages<T> = Box<BTreeMap<T, SharedStr>>;

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
mod violations;
pub use violations::*;
