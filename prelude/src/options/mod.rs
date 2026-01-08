mod common_options;

use std::sync::Arc;

use askama::Template;
use bytes::Bytes;
pub use common_options::*;
use proto_types::{Duration, Timestamp, protovalidate::Ignore};

use crate::*;

#[derive(Clone, Debug, PartialEq)]
pub struct ProtoOption {
  pub name: Arc<str>,
  pub value: OptionValue,
}

/// An enum representing values for protobuf options.
#[derive(Clone, Debug, PartialEq, Template)]
#[template(path = "option_value.proto.j2")]
pub enum OptionValue {
  Bool(bool),
  Int(i64),
  Uint(u64),
  Float(f64),
  String(Arc<str>),
  Bytes(Bytes),
  List(Arc<[Self]>),
  Message(OptionMessage),
  Enum(Arc<str>),
  Duration(Duration),
  Timestamp(Timestamp),
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct OptionMessage {
  inner: Arc<[(Arc<str>, OptionValue)]>,
}

impl<'a> IntoIterator for &'a OptionMessage {
  type Item = &'a (Arc<str>, OptionValue);
  type IntoIter = std::slice::Iter<'a, (Arc<str>, OptionValue)>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.inner.iter()
  }
}

impl<N, V> FromIterator<(N, V)> for OptionMessage
where
  N: Into<Arc<str>>,
  V: Into<OptionValue>,
{
  fn from_iter<T: IntoIterator<Item = (N, V)>>(iter: T) -> Self {
    let items: Vec<(Arc<str>, OptionValue)> = iter
      .into_iter()
      .map(|(n, v)| (n.into(), v.into()))
      .collect();

    Self {
      inner: items.into_boxed_slice().into(),
    }
  }
}

impl OptionMessage {
  #[must_use]
  #[inline]
  pub fn new() -> Self {
    Self::default()
  }

  #[inline]
  #[must_use]
  pub fn get(&self, name: &str) -> Option<&OptionValue> {
    self
      .inner
      .iter()
      .find_map(|(n, v)| (n.as_ref() == name).then_some(v))
  }

  #[inline]
  pub fn iter(&self) -> std::slice::Iter<'_, (Arc<str>, OptionValue)> {
    self.inner.iter()
  }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct OptionMessageBuilder {
  inner: Vec<(Arc<str>, OptionValue)>,
}

impl OptionMessageBuilder {
  pub(crate) fn maybe_set(
    &mut self,
    name: &Arc<str>,
    value: Option<impl Into<OptionValue>>,
  ) -> &mut Self {
    if let Some(value) = value {
      self.set(name.clone(), value);
    }
    self
  }

  pub(crate) fn set_boolean(&mut self, name: &Arc<str>, boolean: bool) -> &mut Self {
    if boolean {
      self.set(name.clone(), OptionValue::Bool(true));
    }
    self
  }

  pub(crate) fn add_cel_options(&mut self, rules: Vec<CelProgram>) -> &mut Self {
    if !rules.is_empty() {
      let rule_options: Vec<OptionValue> = rules
        .into_iter()
        .map(|program| program.rule.into())
        .collect();
      self.set(CEL.clone(), OptionValue::List(rule_options.into()));
    }
    self
  }

  #[inline]
  #[must_use]
  pub fn new() -> Self {
    Self::default()
  }

  #[inline]
  pub fn set(&mut self, name: impl Into<Arc<str>>, value: impl Into<OptionValue>) -> &mut Self {
    self.inner.push((name.into(), value.into()));
    self
  }

  pub(crate) fn set_required(&mut self, required: bool) -> &mut Self {
    if required {
      self.set(REQUIRED.clone(), OptionValue::Bool(true));
    }
    self
  }

  pub(crate) fn set_ignore(&mut self, ignore: Ignore) -> &mut Self {
    if !matches!(ignore, Ignore::Unspecified) {
      // The long conversion is necessary here to avoid issues with the i32 representation
      self.set(IGNORE.clone(), <Ignore as Into<OptionValue>>::into(ignore));
    }
    self
  }

  #[inline]
  pub fn iter(&self) -> std::slice::Iter<'_, KeyValue> {
    self.inner.iter()
  }

  #[inline]
  #[must_use]
  pub fn build(self) -> OptionMessage {
    OptionMessage {
      inner: self.inner.into_boxed_slice().into(),
    }
  }
}

impl From<OptionMessageBuilder> for OptionMessage {
  fn from(value: OptionMessageBuilder) -> Self {
    value.build()
  }
}

type KeyValue = (Arc<str>, OptionValue);

impl IntoIterator for OptionMessageBuilder {
  type Item = KeyValue;
  type IntoIter = std::vec::IntoIter<KeyValue>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.inner.into_iter()
  }
}

impl<'a> IntoIterator for &'a OptionMessageBuilder {
  type Item = &'a KeyValue;
  type IntoIter = std::slice::Iter<'a, KeyValue>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.inner.iter()
  }
}

impl<N, V> FromIterator<(N, V)> for OptionMessageBuilder
where
  N: Into<Arc<str>>,
  V: Into<OptionValue>,
{
  fn from_iter<T: IntoIterator<Item = (N, V)>>(iter: T) -> Self {
    let items: Vec<KeyValue> = iter
      .into_iter()
      .map(|(n, v)| (n.into(), v.into()))
      .collect();

    Self { inner: items }
  }
}

impl<N, V> Extend<(N, V)> for OptionMessageBuilder
where
  N: Into<Arc<str>>,
  V: Into<OptionValue>,
{
  fn extend<T: IntoIterator<Item = (N, V)>>(&mut self, iter: T) {
    self.inner.extend(
      iter
        .into_iter()
        .map(|(n, v)| (n.into(), v.into())),
    )
  }
}

impl OptionValue {
  pub(crate) fn is_short(&self) -> bool {
    match self {
      Self::List(list) => list.len() <= 5 && list.iter().all(Self::is_short),
      Self::String(str) => str.chars().count() <= 5,
      Self::Duration(_) | Self::Timestamp(_) | Self::Message(_) => false,
      _ => true,
    }
  }

  /// Creates a new message option value.
  pub fn new_message<S, V, I>(items: I) -> Self
  where
    S: Into<Arc<str>>,
    V: Into<Self>,
    I: IntoIterator<Item = (S, V)>,
  {
    Self::Message(items.into_iter().collect())
  }

  /// Creates a new list option value.
  pub fn new_list<I, V>(items: I) -> Self
  where
    V: Into<Self>,
    I: IntoIterator<Item = V>,
  {
    let items: Vec<Self> = items
      .into_iter()
      .map(std::convert::Into::into)
      .collect();

    Self::List(items.into())
  }
}

impl<T: Clone + Into<Self>> From<&T> for OptionValue {
  fn from(value: &T) -> Self {
    value.clone().into()
  }
}

macro_rules! option_value_conversion {
  ($origin_type:ty, $dest_type:ident $(, as $as_type:ty)?) => {
    impl From<$origin_type> for OptionValue {
      fn from(value: $origin_type) -> OptionValue {
        OptionValue::$dest_type(value $(as $as_type)?)
      }
    }
  };
}

impl<T: Into<Self> + Clone> From<Arc<[T]>> for OptionValue {
  fn from(value: Arc<[T]>) -> Self {
    Self::List(
      value
        .iter()
        .map(|item| (*item).clone().into())
        .collect::<Vec<Self>>()
        .into(),
    )
  }
}

impl<T: Into<Self> + Clone> From<Vec<T>> for OptionValue {
  fn from(value: Vec<T>) -> Self {
    Self::List(
      value
        .into_iter()
        .map(|item| item.into())
        .collect::<Vec<Self>>()
        .into(),
    )
  }
}

impl<T: Into<Self> + Clone> From<&[T]> for OptionValue {
  fn from(value: &[T]) -> Self {
    Self::List(
      value
        .iter()
        .map(|item| item.clone().into())
        .collect::<Vec<Self>>()
        .into(),
    )
  }
}

impl From<&str> for OptionValue {
  fn from(value: &str) -> Self {
    Self::String(value.into())
  }
}

impl From<Arc<str>> for OptionValue {
  fn from(value: Arc<str>) -> Self {
    Self::String(value)
  }
}

impl From<std::time::Duration> for OptionValue {
  fn from(value: std::time::Duration) -> Self {
    let seconds = value.as_secs().cast_signed();
    #[allow(clippy::cast_possible_truncation)]
    let nanos = value.as_nanos() as i32;

    let duration = Duration { seconds, nanos };

    Self::Duration(duration)
  }
}

impl From<usize> for OptionValue {
  fn from(value: usize) -> Self {
    Self::Uint(value as u64)
  }
}

impl From<isize> for OptionValue {
  fn from(value: isize) -> Self {
    Self::Int(value as i64)
  }
}

option_value_conversion!(bool, Bool);
option_value_conversion!(Duration, Duration);
option_value_conversion!(Timestamp, Timestamp);
option_value_conversion!(i64, Int);
option_value_conversion!(i32, Int, as i64);
option_value_conversion!(u64, Uint);
option_value_conversion!(u32, Uint, as u64);
option_value_conversion!(f64, Float);
option_value_conversion!(f32, Float, as f64);
option_value_conversion!(Bytes, Bytes);
