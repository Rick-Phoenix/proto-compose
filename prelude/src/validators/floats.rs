mod builder;
pub use builder::FloatValidatorBuilder;

use float_eq::FloatEq;
use float_eq::float_eq;

use super::*;

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
  pub in_: Option<StaticLookup<OrderedFloat<Num::RustType>>>,

  /// Specifies that the values in this list will be considered NOT valid for this field.
  pub not_in: Option<StaticLookup<OrderedFloat<Num::RustType>>>,
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
  type UniqueStore<'a>
    = FloatEpsilonStore<Num::RustType>
  where
    Self: 'a;

  #[inline]
  fn make_unique_store<'a>(&self, size: usize) -> Self::UniqueStore<'a>
  where
    Num: 'a,
  {
    FloatEpsilonStore::new(size, self.abs_tolerance, self.rel_tolerance)
  }

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
      self.in_.as_ref().map(|l| l.items.as_slice()),
      self.not_in.as_ref().map(|l| l.items.as_slice()),
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

  fn validate(&self, ctx: &mut ValidationCtx, val: Option<&Self::Target>) {
    handle_ignore_always!(&self.ignore);
    handle_ignore_if_zero_value!(&self.ignore, val.is_none_or(|v| v.is_default()));

    if self.required && val.is_none_or(|v| v.is_default()) {
      ctx.add_required_violation();
    }

    if let Some(&val) = val {
      if let Some(const_val) = self.const_ {
        if !self.float_is_eq(const_val, val) {
          ctx.add_violation(
            Num::CONST_VIOLATION,
            &format!("must be equal to {const_val}"),
          );
        }

        // Using `const` implies no other rules
        return;
      }

      if self.finite && !val.is_finite() {
        ctx.add_violation(Num::FINITE_VIOLATION, "must be a finite number");
      }

      if let Some(gt) = self.gt
        && (val.is_nan() || self.float_is_eq(gt, val) || val < gt)
      {
        ctx.add_violation(Num::GT_VIOLATION, &format!("must be greater than {gt}"));
      }

      if let Some(gte) = self.gte
        && (val.is_nan() || !self.float_is_eq(gte, val) && val < gte)
      {
        ctx.add_violation(
          Num::GTE_VIOLATION,
          &format!("must be greater than or equal to {gte}"),
        );
      }

      if let Some(lt) = self.lt
        && (val.is_nan() || self.float_is_eq(lt, val) || val > lt)
      {
        ctx.add_violation(Num::LT_VIOLATION, &format!("must be smaller than {lt}"));
      }

      if let Some(lte) = self.lte
        && (val.is_nan() || !self.float_is_eq(lte, val) && val > lte)
      {
        ctx.add_violation(
          Num::LTE_VIOLATION,
          &format!("must be smaller than or equal to {lte}"),
        );
      }

      if let Some(allowed_list) = &self.in_
        && !float_in_list(
          val,
          &allowed_list.items,
          self.abs_tolerance,
          self.rel_tolerance,
        )
      {
        let err = ["must be one of these values: ", &allowed_list.items_str].concat();

        ctx.add_violation(Num::IN_VIOLATION, &err);
      }

      if let Some(forbidden_list) = &self.not_in
        && float_in_list(
          val,
          &forbidden_list.items,
          self.abs_tolerance,
          self.rel_tolerance,
        )
      {
        let err = ["cannot be one of these values: ", &forbidden_list.items_str].concat();

        ctx.add_violation(Num::NOT_IN_VIOLATION, &err);
      }

      #[cfg(feature = "cel")]
      if !self.cel.is_empty() {
        let ctx = ProgramsExecutionCtx {
          programs: &self.cel,
          value: val,
          violations: ctx.violations,
          field_context: Some(&ctx.field_context),
          parent_elements: ctx.parent_elements,
        };

        ctx.execute_programs();
      }
    }
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

    outer_rules.set(N::type_name(), OptionValue::Message(rules.into()));

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

pub trait FloatWrapper: AsProtoType + Default {
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
  const LT_VIOLATION: &'static LazyLock<ViolationData>;
  const LTE_VIOLATION: &'static LazyLock<ViolationData>;
  const GT_VIOLATION: &'static LazyLock<ViolationData>;
  const GTE_VIOLATION: &'static LazyLock<ViolationData>;
  const IN_VIOLATION: &'static LazyLock<ViolationData>;
  const NOT_IN_VIOLATION: &'static LazyLock<ViolationData>;
  const CONST_VIOLATION: &'static LazyLock<ViolationData>;
  const FINITE_VIOLATION: &'static LazyLock<ViolationData>;
  #[allow(private_interfaces)]
  const SEALED: Sealed;

  fn type_name() -> &'static str;
}

macro_rules! impl_float_wrapper {
  ($target_type:ty, $proto_type:ident) => {
    paste::paste! {
      impl FloatWrapper for $target_type {
        type RustType = $target_type;
        const LT_VIOLATION: &'static LazyLock<ViolationData> = &[< $proto_type _LT_VIOLATION >];
        const LTE_VIOLATION: &'static LazyLock<ViolationData> = &[< $proto_type _LTE_VIOLATION >];
        const GT_VIOLATION: &'static LazyLock<ViolationData> = &[< $proto_type _GT_VIOLATION >];
        const GTE_VIOLATION: &'static LazyLock<ViolationData> = &[< $proto_type _GTE_VIOLATION >];
        const CONST_VIOLATION: &'static LazyLock<ViolationData> = &[< $proto_type _CONST_VIOLATION >];
        const FINITE_VIOLATION: &'static LazyLock<ViolationData> = &[< $proto_type _FINITE_VIOLATION >];
        const IN_VIOLATION: &'static LazyLock<ViolationData> = &[< $proto_type _IN_VIOLATION >];
        const NOT_IN_VIOLATION: &'static LazyLock<ViolationData> = &[< $proto_type _NOT_IN_VIOLATION >];
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

impl_float_wrapper!(f32, FLOAT);
impl_float_wrapper!(f64, DOUBLE);
