use crate::*;

use proto_types::protovalidate::*;

pub struct FnValidator<F, T: ?Sized> {
  func: F,
  _phantom: PhantomData<T>,
}

impl<F, T> Validator<T> for FnValidator<F, T>
where
  T: ToOwned + ?Sized,
  F: Fn(&mut ValidationCtx, Option<&T>) -> bool,
{
  type Target = T;
  type UniqueStore<'a>
    = UnsupportedStore<T>
  where
    Self: 'a;

  fn validate_core<V>(&self, ctx: &mut ValidationCtx, val: Option<&V>) -> bool
  where
    V: Borrow<Self::Target> + ?Sized,
  {
    let target = val.map(|v| v.borrow());
    (self.func)(ctx, target)
  }
}

pub const fn from_fn<T, F>(f: F) -> FnValidator<F, T>
where
  T: ?Sized,
  F: Fn(&mut ValidationCtx, Option<&T>) -> bool,
{
  FnValidator {
    func: f,
    _phantom: PhantomData,
  }
}

struct Test;

fn validator(ctx: &mut ValidationCtx, val: Option<&str>) -> bool {
  true
}

fn abc() {
  let x = Test.validate("abc");

  let v = from_fn(validator);

  let z = v.validate("abc");
}

impl Validator<String> for Test {
  type Target = str;
  type UniqueStore<'a>
    = RefHybridStore<'a, str>
  where
    Self: 'a;

  fn validate_core<V>(&self, ctx: &mut ValidationCtx, val: Option<&V>) -> bool
  where
    V: Borrow<Self::Target> + ?Sized,
  {
    unimplemented!()
  }
}

// Here we use a generic for the target of the validator
// AND an assoc. type for the actual type being validated
// so that it can be proxied by wrappers (like with Sint32, Fixed32, enums, etc...).
// Same for `ValidatorBuilderFor`.
pub trait Validator<T: ?Sized>: Sized {
  type Target: ToOwned + ?Sized;
  type UniqueStore<'a>: UniqueStore<'a, Item = Self::Target>
  where
    Self: 'a;

  fn make_unique_store<'a>(&self, cap: usize) -> Self::UniqueStore<'a> {
    Self::UniqueStore::default_with_capacity(cap)
  }

  fn cel_rules(&self) -> Vec<CelRule> {
    vec![]
  }

  fn into_proto_option(self) -> Option<ProtoOption> {
    None
  }

  fn into_schema(self) -> Option<FieldValidatorSchema> {
    let cel_rules = self.cel_rules();

    self
      .into_proto_option()
      .map(|opt| FieldValidatorSchema {
        schema: opt,
        cel_rules,
      })
  }

  fn check_consistency(&self) -> Result<(), Vec<ConsistencyError>> {
    Ok(())
  }

  #[cfg(feature = "cel")]
  fn check_cel_programs_with(
    &self,
    _val: <Self::Target as ToOwned>::Owned,
  ) -> Result<(), Vec<CelError>> {
    Ok(())
  }

  #[cfg(feature = "cel")]
  fn check_cel_programs(&self) -> Result<(), Vec<CelError>> {
    Ok(())
  }

  #[inline]
  fn validate<V>(&self, val: &V) -> Result<(), ViolationsAcc>
  where
    V: Borrow<Self::Target> + ?Sized,
  {
    let mut ctx = ValidationCtx::default();

    self.validate_core(&mut ctx, Some(val));

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

    self.validate_core(&mut ctx, val);

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
    self.validate_core(&mut ctx, Some(val));

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
    self.validate_core(&mut ctx, val);

    if ctx.violations.is_empty() {
      Ok(())
    } else {
      Err(ctx.violations)
    }
  }

  fn validate_core<V>(&self, ctx: &mut ValidationCtx, val: Option<&V>) -> bool
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

pub(crate) trait IsDefault: Default + PartialEq {
  fn is_default(&self) -> bool {
    (*self) == Self::default()
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
