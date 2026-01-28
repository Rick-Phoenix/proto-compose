use crate::*;

pub trait ProxiedOneof: From<Self::Proxy> + Into<Self::Proxy> {
  type Proxy: OneofProxy<Oneof = Self> + From<Self> + Into<Self>;

  #[inline]
  fn into_proxy(self) -> Self::Proxy {
    self.into()
  }
}

pub trait OneofProxy: From<Self::Oneof> + Into<Self::Oneof> {
  type Oneof: ProtoOneof + From<Self> + Into<Self>;

  #[inline]
  fn into_oneof(self) -> Self::Oneof {
    self.into()
  }
}

impl<T: OneofProxy> ProtoOneof for T {
  const NAME: &str = T::Oneof::NAME;
  const TAGS: &[i32] = T::Oneof::TAGS;

  fn proto_schema() -> Oneof {
    T::Oneof::proto_schema()
  }
}

pub trait ProtoOneof {
  const NAME: &str;
  const TAGS: &[i32];

  fn proto_schema() -> Oneof;

  #[doc(hidden)]
  fn check_tags(message: &str, found_tags: &mut [i32]) -> Result<(), String> {
    use similar_asserts::SimpleDiff;

    let expected = Self::TAGS;
    let oneof_name = Self::NAME;

    found_tags.sort_unstable();

    if expected != found_tags {
      let exp_str = format!("{expected:#?}");
      let found_str = format!("{found_tags:#?}");

      let diff = SimpleDiff::from_str(&exp_str, &found_str, "expected", "found");

      let error =
        format!("Found tags mismatch for oneof {oneof_name} in message {message}:\n{diff}");

      return Err(error);
    }

    Ok(())
  }
}

pub trait ValidatedOneof: ProtoValidation + Clone {
  fn validate(&self, ctx: &mut ValidationCtx) -> ValidationResult;
}

#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Oneof {
  pub name: FixedStr,
  pub fields: Vec<Field>,
  pub options: Vec<ProtoOption>,
  pub validators: Vec<ValidatorSchema>,
}

impl Oneof {
  pub(crate) fn options_with_validators(&self) -> impl Iterator<Item = &options::ProtoOption> {
    self
      .options
      .iter()
      .chain(self.validators.iter().map(|v| &v.schema))
  }

  #[must_use]
  pub fn with_name(mut self, name: impl Into<FixedStr>) -> Self {
    self.name = name.into();
    self
  }

  #[must_use]
  pub fn with_validators<I: IntoIterator<Item = ValidatorSchema>>(mut self, validators: I) -> Self {
    self.validators.extend(validators);
    self
  }

  #[must_use]
  pub fn with_options<I: IntoIterator<Item = ProtoOption>>(mut self, options: I) -> Self {
    self.options.extend(options);
    self
  }
}
