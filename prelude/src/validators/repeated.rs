use std::marker::PhantomData;

use bon::Builder;
use repeated_validator_builder::{IsComplete, IsUnset, SetIgnore, State};

use super::*;

pub struct ProtoRepeated<T>(PhantomData<T>);

macro_rules! impl_repeated {
  ($name:ident) => {
    impl_repeated_validator!($name);

    impl<T: AsProtoType> AsProtoType for $name<T> {
      fn proto_type() -> ProtoType {
        let inner_type = T::proto_type();

        match inner_type {
          ProtoType::Single(data) => ProtoType::Repeated(data),
          _ => ProtoType::Repeated(invalid_type_output(
            "Repeated fields cannot be optional, maps or other repeated fields",
          )),
        }
      }
    }
  };
}

macro_rules! impl_repeated_validator {
  ($name:ident) => {
    impl<T> ProtoValidator<$name<T>> for ValidatorMap
    where
      ValidatorMap: ProtoValidator<T>,
      T: AsProtoType,
    {
      type Builder = RepeatedValidatorBuilder<T>;

      fn builder() -> Self::Builder {
        RepeatedValidator::builder()
      }
    }

    impl<T: AsProtoType, S: State> ValidatorBuilderFor<$name<T>>
      for RepeatedValidatorBuilder<T, S>
    {
    }
  };
}

impl_repeated!(ProtoRepeated);
impl_repeated!(Vec);

impl<T: AsProtoType, S: State> RepeatedValidatorBuilder<T, S>
where
  S::Items: repeated_validator_builder::IsUnset,
{
  /// Specifies the rules that will be applied to the individual items of this repeated field.
  pub fn items<F, FinalBuilder>(
    self,
    config_fn: F,
  ) -> RepeatedValidatorBuilder<T, repeated_validator_builder::SetItems<S>>
  where
    F: FnOnce(<ValidatorMap as ProtoValidator<T>>::Builder) -> FinalBuilder,
    FinalBuilder: ValidatorBuilderFor<T>,
    ValidatorMap: ProtoValidator<T>,
  {
    let builder = ValidatorMap::builder();
    let options = config_fn(builder).into();
    self.items_internal(options)
  }
}

impl<S: State, T: AsProtoType> RepeatedValidatorBuilder<T, S>
where
  S::Ignore: IsUnset,
{
  /// Rules set for this field will always be ignored.
  pub fn ignore_always(self) -> RepeatedValidatorBuilder<T, SetIgnore<S>> {
    self.ignore(Ignore::Always)
  }
}

#[derive(Clone, Debug, Builder)]
#[builder(state_mod(vis = "pub"))]
#[builder(derive(Clone))]
pub struct RepeatedValidator<T: AsProtoType> {
  #[builder(default, setters(vis = ""))]
  _inner_type: PhantomData<T>,

  /// Specifies the rules that will be applied to the individual items of this repeated field.
  #[builder(setters(vis = "", name = items_internal))]
  pub items: Option<ProtoOption>,
  /// The minimum amount of items that this field must contain in order to be valid.
  pub min_items: Option<u64>,
  /// The maximum amount of items that this field must contain in order to be valid.
  pub max_items: Option<u64>,
  #[builder(with = || true)]
  /// Specifies that this field must contain only unique values (only applies to scalar fields).
  pub unique: Option<bool>,
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  /// These will apply to the list as a whole. To apply rules to the individual items, use the items validator instead.
  #[builder(into)]
  pub cel: Option<Arc<[CelRule]>>,
  /// Specifies that the field must be set in order to be valid. This is essentially the same as setting min_items to 1
  #[builder(with = || true)]
  pub required: Option<bool>,
  #[builder(setters(vis = "", name = ignore))]
  pub ignore: Option<Ignore>,
}

impl<T: AsProtoType, S: State> From<RepeatedValidatorBuilder<T, S>> for ProtoOption
where
  S: IsComplete,
{
  fn from(value: RepeatedValidatorBuilder<T, S>) -> Self {
    value.build().into()
  }
}

impl<T: AsProtoType> From<RepeatedValidator<T>> for ProtoOption {
  fn from(validator: RepeatedValidator<T>) -> ProtoOption {
    let mut rules: OptionValueList = Vec::new();

    insert_option!(validator, rules, unique);
    insert_option!(validator, rules, min_items);
    insert_option!(validator, rules, max_items);

    if let Some(items_option) = validator.items {
      rules.push((ITEMS.clone(), items_option.value.clone()));
    }

    let mut outer_rules: OptionValueList = vec![];

    outer_rules.push((REPEATED.clone(), OptionValue::Message(rules.into())));

    insert_cel_rules!(validator, outer_rules);
    insert_option!(validator, outer_rules, required);
    insert_option!(validator, outer_rules, ignore);

    ProtoOption {
      name: BUF_VALIDATE_FIELD.clone(),
      value: OptionValue::Message(outer_rules.into()),
    }
  }
}
