mod builder;
pub use builder::IntValidatorBuilder;

pub use proto_types::num_wrappers::*;
use proto_types::protovalidate::violations_data::*;

use super::*;

#[non_exhaustive]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IntValidator<Num>
where
  Num: IntWrapper,
{
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  pub cel: Vec<CelProgram>,

  pub ignore: Ignore,

  #[cfg_attr(feature = "serde", serde(skip))]
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
  pub in_: Option<SortedList<Num::RustType>>,

  /// Specifies that the values in this list will be considered NOT valid for this field.
  pub not_in: Option<SortedList<Num::RustType>>,

  pub error_messages: Option<ErrorMessages<Num::ViolationEnum>>,
}

impl<Num> Default for IntValidator<Num>
where
  Num: IntWrapper + Default,
{
  #[inline]
  fn default() -> Self {
    Self {
      cel: Default::default(),
      ignore: Default::default(),
      _wrapper: Default::default(),
      required: Default::default(),
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

impl<Num> IntValidator<Num>
where
  Num: IntWrapper,
{
  #[inline(never)]
  #[cold]
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

impl<S: builder::state::State, Num: IntWrapper> ValidatorBuilderFor<Num>
  for IntValidatorBuilder<Num, S>
{
  type Target = Num::RustType;
  type Validator = IntValidator<Num>;

  #[inline]
  fn build_validator(self) -> Self::Validator {
    self.build()
  }
}

impl<Num> Validator<Num> for IntValidator<Num>
where
  Num: IntWrapper,
{
  type Target = Num::RustType;

  impl_testing_methods!();

  #[inline(never)]
  #[cold]
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

    if let Some(custom_messages) = self.error_messages.as_deref() {
      let mut unused_messages: Vec<String> = Vec::new();

      for key in custom_messages.keys() {
        macro_rules! check_unused_messages {
          ($($name:ident),*) => {
            paste! {
              $(
                (*key == Num::[< $name:snake:upper _VIOLATION >] && self.$name.is_some())
              ) ||*
            }
          };
        }

        let is_used = check_unused_messages!(gt, gte, lt, lte, not_in)
          || (*key == Num::REQUIRED_VIOLATION && self.required)
          || (*key == Num::CONST_VIOLATION && self.const_.is_some())
          || (*key == Num::IN_VIOLATION && self.in_.is_some());

        if !is_used {
          unused_messages.push(format!("{key:?}"));
        }
      }

      if !unused_messages.is_empty() {
        errors.push(ConsistencyError::UnusedCustomMessages(unused_messages));
      }
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

  fn validate_core<V>(&self, ctx: &mut ValidationCtx, val: Option<&V>) -> ValidationResult
  where
    V: Borrow<Self::Target> + ?Sized,
  {
    handle_ignore_always!(&self.ignore);
    handle_ignore_if_zero_value!(&self.ignore, val.is_none_or(|v| v.borrow().is_default()));

    let mut is_valid = IsValid::Yes;

    macro_rules! handle_violation {
      ($id:ident, $default:expr) => {
        paste::paste! {
          is_valid &= ctx.add_violation(
            Num::[< $id:snake:upper _VIOLATION >].into(),
            self.custom_error_or_else(
              Num::[< $id:snake:upper _VIOLATION >],
              || $default
            )
          )?;
        }
      };
    }

    if self.required && val.is_none_or(|v| v.borrow().is_default()) {
      handle_violation!(Required, "is required".to_string());
      return Ok(is_valid);
    }

    if let Some(val) = val {
      let val = *val.borrow();

      if let Some(const_val) = self.const_ {
        if val != const_val {
          handle_violation!(Const, format!("must be equal to {const_val}"));
        }

        // Using `const` implies no other rules
        return Ok(is_valid);
      }

      if let Some(gt) = self.gt
        && val <= gt
      {
        handle_violation!(Gt, format!("must be greater than {gt}"));
      }

      if let Some(gte) = self.gte
        && val < gte
      {
        handle_violation!(Gte, format!("must be greater than or equal to {gte}"));
      }

      if let Some(lt) = self.lt
        && val >= lt
      {
        handle_violation!(Lt, format!("must be smaller than {lt}"));
      }

      if let Some(lte) = self.lte
        && val > lte
      {
        handle_violation!(Lte, format!("must be smaller than or equal to {lte}"));
      }

      if let Some(allowed_list) = &self.in_
        && !allowed_list.contains(&val)
      {
        handle_violation!(
          In,
          format!(
            "must be one of these values: {}",
            Num::RustType::format_list(allowed_list)
          )
        );
      }

      if let Some(forbidden_list) = &self.not_in
        && forbidden_list.contains(&val)
      {
        handle_violation!(
          NotIn,
          format!(
            "cannot be one of these values: {}",
            Num::RustType::format_list(forbidden_list)
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

        is_valid &= cel_ctx.execute_programs()?;
      }
    }

    Ok(is_valid)
  }

  #[inline(never)]
  #[cold]
  fn schema(&self) -> Option<ValidatorSchema> {
    Some(ValidatorSchema {
      schema: self.clone().into(),
      cel_rules: self.cel_rules(),
      imports: vec!["buf/validate/validate.proto".into()],
    })
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
  #[inline(never)]
  #[cold]
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
          .map(|list| OptionValue::new_list(list)),
      )
      .maybe_set(
        "not_in",
        validator
          .not_in
          .map(|list| OptionValue::new_list(list)),
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

#[allow(private_interfaces)]
struct Sealed;

pub trait IntWrapper: AsProtoType + Default + Copy + Send + Sync {
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
    + Send
    + Sync
    + MaybeSerde
    + 'static;
  type ViolationEnum: Copy + Ord + Into<ViolationKind> + Debug + Send + Sync + MaybeSerde;
  const LT_VIOLATION: Self::ViolationEnum;
  const LTE_VIOLATION: Self::ViolationEnum;
  const GT_VIOLATION: Self::ViolationEnum;
  const GTE_VIOLATION: Self::ViolationEnum;
  const IN_VIOLATION: Self::ViolationEnum;
  const NOT_IN_VIOLATION: Self::ViolationEnum;
  const CONST_VIOLATION: Self::ViolationEnum;
  const REQUIRED_VIOLATION: Self::ViolationEnum;
  #[allow(private_interfaces)]
  const SEALED: Sealed;

  fn type_name() -> &'static str;
}

macro_rules! impl_int_wrapper {
  ($wrapper:ty, $target_type:ty, $proto_type:ident) => {
    paste::paste! {
      impl IntWrapper for $wrapper {
        #[doc(hidden)]
        type RustType = $target_type;
        #[doc(hidden)]
        type ViolationEnum = [< $proto_type Violation >];
        #[doc(hidden)]
        const LT_VIOLATION: Self::ViolationEnum = [< $proto_type Violation >]::Lt;
        #[doc(hidden)]
        const LTE_VIOLATION: Self::ViolationEnum = [< $proto_type Violation >]::Lte;
        #[doc(hidden)]
        const GT_VIOLATION: Self::ViolationEnum = [< $proto_type Violation >]::Gt;
        #[doc(hidden)]
        const GTE_VIOLATION: Self::ViolationEnum = [< $proto_type Violation >]::Gte;
        #[doc(hidden)]
        const CONST_VIOLATION: Self::ViolationEnum = [< $proto_type Violation >]::Const;
        #[doc(hidden)]
        const IN_VIOLATION: Self::ViolationEnum = [< $proto_type Violation >]::In;
        #[doc(hidden)]
        const NOT_IN_VIOLATION: Self::ViolationEnum = [< $proto_type Violation >]::NotIn;
        #[doc(hidden)]
        const REQUIRED_VIOLATION: Self::ViolationEnum = [< $proto_type Violation >]::Required;
        #[doc(hidden)]
        #[allow(private_interfaces)]
        const SEALED: Sealed = Sealed;

        #[doc(hidden)]
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
    impl_int_wrapper!($wrapper, $rust_type, $wrapper);
    impl_proto_type!($wrapper, $wrapper);
    impl_proto_map_key!($wrapper, $wrapper);
    impl_int_validator!($wrapper, $rust_type);
  };
}

macro_rules! impl_int_validator {
  ($wrapper:ty, $rust_type:ty) => {
    $crate::paste! {
      impl ProtoValidation for $wrapper {
        #[doc(hidden)]
        type Target = $rust_type;
        #[doc(hidden)]
        type Stored = $rust_type;
        type Validator = IntValidator<$wrapper>;
        #[doc(hidden)]
        type Builder = IntValidatorBuilder<$wrapper>;

        type UniqueStore<'a>
          = CopyHybridStore<$rust_type>
        where
          Self: 'a;

        #[inline]
        fn make_unique_store<'a>(
          _: &Self::Validator,
          cap: usize,
        ) -> Self::UniqueStore<'a>
        {
          CopyHybridStore::default_with_capacity(cap)
        }
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
impl_int!(i32, Int32, primitive);
impl_int!(i64, Int64, primitive);
impl_int!(u32, Uint32, primitive);
impl_int!(u64, Uint64, primitive);
