mod builder;
pub use builder::FloatValidatorBuilder;

use float_eq::FloatEq;
use float_eq::float_eq;

use super::*;

#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct FloatValidator<Num>
where
  Num: FloatWrapper,
{
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  pub cel: Vec<CelProgram>,

  pub ignore: Ignore,

  _wrapper: PhantomData<Num>,

  /// Specifies that the field must be set in order to be valid.
  pub required: bool,

  /// The absolute tolerance to use for equality operations
  pub abs_tolerance: Num::RustType,

  /// The relative tolerance to use for equality operations, scaled to the precision of the number being validated
  pub rel_tolerance: Num::RustType,

  /// Specifies that this field must be finite (i.e. it can't represent Infinity or NaN)
  pub finite: bool,

  /// Specifies that only this specific value will be considered valid for this field.
  pub const_: Option<Num::RustType>,

  /// Specifies that this field's value will be valid only if it is smaller than the specified amount
  pub lt: Option<Num::RustType>,

  /// Specifies that this field's value will be valid only if it is smaller than, or equal to, the specified amount
  pub lte: Option<Num::RustType>,

  /// Specifies that this field's value will be valid only if it is greater than the specified amount
  pub gt: Option<Num::RustType>,

  /// Specifies that this field's value will be valid only if it is smaller than, or equal to, the specified amount
  pub gte: Option<Num::RustType>,

  /// Specifies that only the values in this list will be considered valid for this field.
  pub in_: Option<SortedList<OrderedFloat<Num::RustType>>>,

  /// Specifies that the values in this list will be considered NOT valid for this field.
  pub not_in: Option<SortedList<OrderedFloat<Num::RustType>>>,

  pub error_messages: Option<ErrorMessages<Num::ViolationEnum>>,
}

impl<Num> FloatValidator<Num>
where
  Num: FloatWrapper,
{
  fn custom_error_or_else(
    &self,
    violation: Num::ViolationEnum,
    default: impl Fn() -> String,
  ) -> String {
    self
      .error_messages
      .as_deref()
      .and_then(|map| map.get(&violation))
      .map(|m| m.to_string())
      .unwrap_or_else(default)
  }
}

impl<Num> Default for FloatValidator<Num>
where
  Num: FloatWrapper + Default,
{
  fn default() -> Self {
    Self {
      cel: Default::default(),
      ignore: Default::default(),
      _wrapper: Default::default(),
      required: Default::default(),
      abs_tolerance: Default::default(),
      rel_tolerance: Default::default(),
      finite: Default::default(),
      const_: Default::default(),
      lt: Default::default(),
      lte: Default::default(),
      gt: Default::default(),
      gte: Default::default(),
      in_: Default::default(),
      not_in: Default::default(),
      error_messages: Default::default(),
    }
  }
}

pub(crate) fn float_in_list<T>(target: T, list: &[OrderedFloat<T>], abs_tol: T, r2nd_tol: T) -> bool
where
  T: FloatCore + FloatEq<Tol = T>,
{
  let wrapped_target: OrderedFloat<T> = target.into();

  // 1. Perform Binary Search
  match list.binary_search(&wrapped_target) {
    // Exact bit-for-bit match found
    Ok(_) => true,

    // No exact match. 'idx' is the insertion point.
    Err(idx) => {
      // 2. Check the neighbor immediately AFTER the insertion point
      if let Some(above) = list.get(idx)
        && float_eq!(above.0, target, abs <= abs_tol, r2nd <= r2nd_tol)
      {
        return true;
      }

      // 3. Check the neighbor immediately BEFORE the insertion point
      if idx > 0
        && let Some(below) = list.get(idx - 1)
        && float_eq!(below.0, target, abs <= abs_tol, r2nd <= r2nd_tol)
      {
        return true;
      }

      false
    }
  }
}

pub(crate) fn check_float_list_rules<T>(
  in_list: Option<&[OrderedFloat<T>]>,
  not_in_list: Option<&[OrderedFloat<T>]>,
  abs_tol: T,
  r2nd_tol: T,
) -> Result<(), OverlappingListsError>
where
  T: FloatCore + Debug + FloatEq<Tol = T>,
{
  if let Some(in_list) = in_list
    && let Some(not_in_list) = not_in_list
  {
    let mut overlapping: Vec<T> = Vec::with_capacity(in_list.len());

    for item in in_list {
      let is_overlapping = float_in_list(item.0, not_in_list, abs_tol, r2nd_tol);

      if is_overlapping {
        overlapping.push(**item);
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

impl<Num> Validator<Num> for FloatValidator<Num>
where
  Num: FloatWrapper,
{
  type Target = Num::RustType;

  impl_testing_methods!();

  fn check_consistency(&self) -> Result<(), Vec<ConsistencyError>> {
    let mut errors = Vec::new();

    macro_rules! check_prop_some {
      ($($id:ident),*) => {
        $(self.$id.is_some()) ||*
      };
    }

    if self.const_.is_some()
      && (!self.cel.is_empty() || self.finite || check_prop_some!(in_, not_in, lt, lte, gt, gte))
    {
      errors.push(ConsistencyError::ConstWithOtherRules);
    }

    #[cfg(feature = "cel")]
    if let Err(e) = self.check_cel_programs() {
      errors.extend(e.into_iter().map(ConsistencyError::from));
    }

    if let Err(e) = check_float_list_rules(
      self.in_.as_deref(),
      self.not_in.as_deref(),
      self.abs_tolerance,
      self.rel_tolerance,
    ) {
      errors.push(e.into());
    }

    if let Err(e) = check_comparable_rules(self.lt, self.lte, self.gt, self.gte) {
      errors.push(e);
    }

    if errors.is_empty() {
      Ok(())
    } else {
      Err(errors)
    }
  }

  fn validate_core<V>(&self, ctx: &mut ValidationCtx, val: Option<&V>) -> bool
  where
    V: Borrow<Self::Target> + ?Sized,
  {
    handle_ignore_always!(&self.ignore);
    handle_ignore_if_zero_value!(&self.ignore, val.is_none_or(|v| v.borrow().is_default()));

    let mut is_valid = true;

    if self.required && val.is_none_or(|v| v.borrow().is_default()) {
      ctx.add_required_violation();
      return false;
    }

    if let Some(val) = val {
      let val = *val.borrow();

      macro_rules! handle_violation {
        ($id:ident, $default:expr) => {
          paste::paste! {
            ctx.add_violation(
              Num::[< $id:snake:upper _VIOLATION >].into(),
              self.custom_error_or_else(
                Num::[< $id:snake:upper _VIOLATION >],
                || $default
              )
            );

            if ctx.fail_fast {
              return false;
            } else {
              is_valid = false;
            }
          }
        };
      }

      if let Some(const_val) = self.const_ {
        if !self.float_is_eq(const_val, val) {
          handle_violation!(Const, format!("must be equal to {const_val}"));
        }

        // Using `const` implies no other rules
        return is_valid;
      }

      if self.finite && !val.is_finite() {
        handle_violation!(Finite, "must be a finite number".to_string());
      }

      if let Some(gt) = self.gt
        && (val.is_nan() || self.float_is_eq(gt, val) || val < gt)
      {
        handle_violation!(Gt, format!("must be greater than {gt}"));
      }

      if let Some(gte) = self.gte
        && (val.is_nan() || !self.float_is_eq(gte, val) && val < gte)
      {
        handle_violation!(Gte, format!("must be greater than or equal to {gte}"));
      }

      if let Some(lt) = self.lt
        && (val.is_nan() || self.float_is_eq(lt, val) || val > lt)
      {
        handle_violation!(Lt, format!("must be smaller than {lt}"));
      }

      if let Some(lte) = self.lte
        && (val.is_nan() || !self.float_is_eq(lte, val) && val > lte)
      {
        handle_violation!(Lte, format!("must be smaller than or equal to {lte}"));
      }

      if let Some(allowed_list) = &self.in_
        && !float_in_list(val, allowed_list, self.abs_tolerance, self.rel_tolerance)
      {
        handle_violation!(
          In,
          format!(
            "must be one of these values: {}",
            OrderedFloat::<Num::RustType>::format_list(allowed_list)
          )
        );
      }

      if let Some(forbidden_list) = &self.not_in
        && float_in_list(val, forbidden_list, self.abs_tolerance, self.rel_tolerance)
      {
        handle_violation!(
          NotIn,
          format!(
            "cannot be one of these values: {}",
            OrderedFloat::<Num::RustType>::format_list(forbidden_list)
          )
        );
      }

      #[cfg(feature = "cel")]
      if !self.cel.is_empty() {
        let cel_ctx = ProgramsExecutionCtx {
          programs: &self.cel,
          value: val,
          ctx,
        };

        is_valid = cel_ctx.execute_programs();
      }
    }

    is_valid
  }

  fn as_proto_option(&self) -> Option<ProtoOption> {
    Some(self.clone().into())
  }
}

impl<Num> FloatValidator<Num>
where
  Num: FloatWrapper,
{
  #[must_use]
  #[inline]
  pub fn builder() -> FloatValidatorBuilder<Num> {
    FloatValidatorBuilder::default()
  }

  #[inline]
  fn float_is_eq(&self, first: Num::RustType, second: Num::RustType) -> bool {
    float_eq!(
      first,
      second,
      abs <= self.abs_tolerance,
      r2nd <= self.rel_tolerance
    )
  }
}

impl<N> From<FloatValidator<N>> for ProtoOption
where
  N: FloatWrapper,
{
  fn from(validator: FloatValidator<N>) -> Self {
    let mut rules = OptionMessageBuilder::new();

    macro_rules! set_options {
      ($($name:ident),*) => {
        rules
        $(
          .maybe_set(stringify!($name), validator.$name)
        )*
      };
    }

    set_options!(lt, lte, gt, gte);

    rules
      .maybe_set("const", validator.const_)
      .set_boolean("finite", validator.finite)
      .maybe_set(
        "in",
        validator
          .in_
          .map(|list| OptionValue::List(list.items.iter().map(|of| of.0).collect())),
      )
      .maybe_set(
        "not_in",
        validator
          .not_in
          .map(|list| OptionValue::List(list.items.iter().map(|of| of.0).collect())),
      );

    let mut outer_rules = OptionMessageBuilder::new();

    if !rules.is_empty() {
      outer_rules.set(N::type_name(), OptionValue::Message(rules.into()));
    }

    outer_rules
      .add_cel_options(validator.cel)
      .set_required(validator.required)
      .set_ignore(validator.ignore);

    Self {
      name: "(buf.validate.field)".into(),
      value: OptionValue::Message(outer_rules.into()),
    }
  }
}

impl_proto_type!(f32, Float);
impl_proto_type!(f64, Double);

#[allow(private_interfaces)]
struct Sealed;

pub trait FloatWrapper: AsProtoType + Default + Copy {
  type RustType: PartialOrd
    + PartialEq
    + Copy
    + Into<OptionValue>
    + Debug
    + Display
    + Default
    + IntoCel
    + ordered_float::FloatCore
    + ordered_float::PrimitiveFloat
    + float_eq::FloatEq<Tol = Self::RustType>
    + 'static;
  type ViolationEnum: Copy + Ord + Into<ViolationKind> + Debug;
  const LT_VIOLATION: Self::ViolationEnum;
  const LTE_VIOLATION: Self::ViolationEnum;
  const GT_VIOLATION: Self::ViolationEnum;
  const GTE_VIOLATION: Self::ViolationEnum;
  const IN_VIOLATION: Self::ViolationEnum;
  const NOT_IN_VIOLATION: Self::ViolationEnum;
  const CONST_VIOLATION: Self::ViolationEnum;
  const FINITE_VIOLATION: Self::ViolationEnum;
  #[allow(private_interfaces)]
  const SEALED: Sealed;

  fn type_name() -> &'static str;
}

macro_rules! impl_float_wrapper {
  ($target_type:ty, $proto_type:ident) => {
    paste::paste! {
      impl FloatWrapper for $target_type {
        type RustType = $target_type;
        type ViolationEnum = [< $proto_type Violation >];
        const LT_VIOLATION: Self::ViolationEnum = [< $proto_type Violation >]::Lt;
        const LTE_VIOLATION: Self::ViolationEnum = [< $proto_type Violation >]::Lte;
        const GT_VIOLATION: Self::ViolationEnum = [< $proto_type Violation >]::Gt;
        const GTE_VIOLATION: Self::ViolationEnum = [< $proto_type Violation >]::Gte;
        const CONST_VIOLATION: Self::ViolationEnum = [< $proto_type Violation >]::Const;
        const FINITE_VIOLATION: Self::ViolationEnum = [< $proto_type Violation >]::Finite;
        const IN_VIOLATION: Self::ViolationEnum = [< $proto_type Violation >]::In;
        const NOT_IN_VIOLATION: Self::ViolationEnum = [< $proto_type Violation >]::NotIn;
        #[allow(private_interfaces)]
        const SEALED: Sealed = Sealed;

        fn type_name() -> &'static str {
          stringify!([< $proto_type:lower >])
        }
      }

      impl ProtoValidator for $target_type {
        type Target = $target_type;
        type Validator = FloatValidator<$target_type>;
        type Builder = FloatValidatorBuilder<$target_type>;

        type UniqueStore<'a>
          = FloatEpsilonStore<$target_type>
        where
          Self: 'a;

        #[inline]
        fn make_unique_store<'a>(
          validator: &Self::Validator,
          size: usize,
        ) -> Self::UniqueStore<'a>
        {
          FloatEpsilonStore::new(size, validator.abs_tolerance, validator.rel_tolerance)
        }
      }

      impl<S: builder::state::State> ValidatorBuilderFor<$target_type> for FloatValidatorBuilder<$target_type, S> {
        type Target = $target_type;
        type Validator = FloatValidator<$target_type>;

        #[inline]
        fn build_validator(self) -> Self::Validator {
          self.build()
        }
      }
    }
  };
}

impl_float_wrapper!(f32, Float);
impl_float_wrapper!(f64, Double);
