mod builder;
pub use builder::IntValidatorBuilder;

pub use proto_types::num_wrappers::*;
use proto_types::protovalidate::violations_data::*;

use super::*;

#[non_exhaustive]
#[derive(Clone, Debug, Default)]
pub struct IntValidator<Num>
where
  Num: IntWrapper,
{
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  pub cel: Vec<CelProgram>,

  pub ignore: Ignore,

  _wrapper: PhantomData<Num>,

  /// Specifies that the field must be set in order to be valid.
  pub required: bool,

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
  pub in_: Option<StaticLookup<Num::RustType>>,

  /// Specifies that the values in this list will be considered NOT valid for this field.
  pub not_in: Option<StaticLookup<Num::RustType>>,
}

impl<S: builder::state::State, Num: IntWrapper> ValidatorBuilderFor<Num>
  for IntValidatorBuilder<Num, S>
{
  type Target = Num::RustType;
  type Validator = IntValidator<Num>;

  #[inline]
  #[doc(hidden)]
  fn build_validator(self) -> Self::Validator {
    self.build()
  }
}

impl<Num> Validator<Num> for IntValidator<Num>
where
  Num: IntWrapper,
{
  type Target = Num::RustType;
  type UniqueStore<'a>
    = CopyHybridStore<Num::RustType>
  where
    Self: 'a;

  #[inline]
  fn make_unique_store<'a>(&self, cap: usize) -> Self::UniqueStore<'a>
  where
    Num: 'a,
  {
    CopyHybridStore::default_with_capacity(cap)
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
      && (!self.cel.is_empty() || check_prop_some!(in_, not_in, lt, lte, gt, gte))
    {
      errors.push(ConsistencyError::ConstWithOtherRules);
    }

    #[cfg(feature = "cel")]
    if let Err(e) = self.check_cel_programs() {
      errors.extend(e.into_iter().map(ConsistencyError::from));
    }

    if let Err(e) = check_list_rules(self.in_.as_ref(), self.not_in.as_ref()) {
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

  fn validate(&self, ctx: &mut ValidationCtx, val: Option<&Self::Target>) -> bool {
    handle_ignore_always!(&self.ignore);
    handle_ignore_if_zero_value!(&self.ignore, val.is_none_or(|v| v.is_default()));

    let mut is_valid = true;

    if self.required && val.is_none_or(|v| v.is_default()) {
      ctx.add_required_violation();
      return false;
    }

    if let Some(&val) = val {
      if let Some(const_val) = self.const_ {
        if val != const_val {
          ctx.add_violation(
            Num::CONST_VIOLATION,
            &format!("must be equal to {const_val}"),
          );

          is_valid = false;
        }

        // Using `const` implies no other rules
        return is_valid;
      }

      if let Some(gt) = self.gt
        && val <= gt
      {
        ctx.add_violation(Num::GT_VIOLATION, &format!("must be greater than {gt}"));
        handle_violation!(is_valid, ctx);
      }

      if let Some(gte) = self.gte
        && val < gte
      {
        ctx.add_violation(
          Num::GTE_VIOLATION,
          &format!("must be greater than or equal to {gte}"),
        );
        handle_violation!(is_valid, ctx);
      }

      if let Some(lt) = self.lt
        && val >= lt
      {
        ctx.add_violation(Num::LT_VIOLATION, &format!("must be smaller than {lt}"));
        handle_violation!(is_valid, ctx);
      }

      if let Some(lte) = self.lte
        && val > lte
      {
        ctx.add_violation(
          Num::LTE_VIOLATION,
          &format!("must be smaller than or equal to {lte}"),
        );
        handle_violation!(is_valid, ctx);
      }

      if let Some(allowed_list) = &self.in_
        && !allowed_list.items.contains(&val)
      {
        let err = ["must be one of these values: ", &allowed_list.items_str].concat();

        ctx.add_violation(Num::IN_VIOLATION, &err);
        handle_violation!(is_valid, ctx);
      }

      if let Some(forbidden_list) = &self.not_in
        && forbidden_list.items.contains(&val)
      {
        let err = ["cannot be one of these values: ", &forbidden_list.items_str].concat();

        ctx.add_violation(Num::NOT_IN_VIOLATION, &err);
        handle_violation!(is_valid, ctx);
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
}

impl<Num> IntValidator<Num>
where
  Num: IntWrapper,
{
  #[must_use]
  #[inline]
  pub fn builder() -> IntValidatorBuilder<Num> {
    IntValidatorBuilder::default()
  }
}

impl<N> From<IntValidator<N>> for ProtoOption
where
  N: IntWrapper,
{
  fn from(validator: IntValidator<N>) -> Self {
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
      .maybe_set(
        "in",
        validator
          .in_
          .map(|list| OptionValue::new_list(list.items)),
      )
      .maybe_set(
        "not_in",
        validator
          .not_in
          .map(|list| OptionValue::new_list(list.items)),
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

#[allow(private_interfaces)]
struct Sealed;

pub trait IntWrapper: AsProtoType + Default {
  type RustType: PartialOrd
    + PartialEq
    + Copy
    + Into<OptionValue>
    + Hash
    + Debug
    + Display
    + Eq
    + Default
    + Ord
    + IntoCel
    + ListFormatter
    + AsProtoMapKey
    + 'static;
  const LT_VIOLATION: ViolationData;
  const LTE_VIOLATION: ViolationData;
  const GT_VIOLATION: ViolationData;
  const GTE_VIOLATION: ViolationData;
  const IN_VIOLATION: ViolationData;
  const NOT_IN_VIOLATION: ViolationData;
  const CONST_VIOLATION: ViolationData;
  #[allow(private_interfaces)]
  const SEALED: Sealed;

  fn type_name() -> &'static str;
}

macro_rules! impl_int_wrapper {
  ($wrapper:ty, $target_type:ty, $proto_type:ident) => {
    paste::paste! {
      impl IntWrapper for $wrapper {
        type RustType = $target_type;
        const LT_VIOLATION: ViolationData = [< $proto_type _LT_VIOLATION >];
        const LTE_VIOLATION: ViolationData = [< $proto_type _LTE_VIOLATION >];
        const GT_VIOLATION: ViolationData = [< $proto_type _GT_VIOLATION >];
        const GTE_VIOLATION: ViolationData = [< $proto_type _GTE_VIOLATION >];
        const IN_VIOLATION: ViolationData = [< $proto_type _IN_VIOLATION >];
        const NOT_IN_VIOLATION: ViolationData = [< $proto_type _NOT_IN_VIOLATION >];
        const CONST_VIOLATION: ViolationData = [< $proto_type _CONST_VIOLATION >];
        #[allow(private_interfaces)]
        const SEALED: Sealed = Sealed;

        fn type_name() -> &'static str {
          stringify!([< $proto_type:lower >])
        }
      }
    }
  };
}

macro_rules! impl_int {
  ($rust_type:ty, $proto_type:ident, primitive) => {
    paste::paste! {
      impl_int_wrapper!($rust_type, $rust_type, $proto_type);
      impl_proto_type!($rust_type, [ < $proto_type:camel > ]);
      impl_proto_map_key!($rust_type, [ < $proto_type:camel > ]);
      impl_int_validator!($rust_type, $rust_type);
    }
  };

  ($rust_type:ty, $wrapper:ident) => {
    paste::paste! {
      impl_int_wrapper!($wrapper, $rust_type, [< $wrapper:upper >]);
      impl_proto_type!($wrapper, $wrapper);
      impl_proto_map_key!($wrapper, $wrapper);
      impl_int_validator!($wrapper, $rust_type);
    }
  };
}

macro_rules! impl_int_validator {
  ($wrapper:ty, $rust_type:ty) => {
    $crate::paste! {
      impl ProtoValidator for $wrapper {
        type Target = $rust_type;
        type Validator = IntValidator<$wrapper>;
        type Builder = IntValidatorBuilder<$wrapper>;
      }
    }
  };
}

impl_int!(i32, Sint32);
impl_int!(i64, Sint64);
impl_int!(i32, Sfixed32);
impl_int!(i64, Sfixed64);
impl_int!(u32, Fixed32);
impl_int!(u64, Fixed64);
impl_int!(i32, INT32, primitive);
impl_int!(i64, INT64, primitive);
impl_int!(u32, UINT32, primitive);
impl_int!(u64, UINT64, primitive);
