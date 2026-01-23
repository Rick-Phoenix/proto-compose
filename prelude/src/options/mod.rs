mod common_options;

use ::bytes::Bytes;
pub use common_options::*;
use proto_types::{Duration, Timestamp, protovalidate::Ignore};

use crate::*;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProtoOption {
  pub name: FixedStr,
  pub value: OptionValue,
}

/// An enum representing values for protobuf options.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "std", derive(Template))]
#[cfg_attr(feature = "std", template(path = "option_value.proto.j2"))]
pub enum OptionValue {
  Bool(bool),
  Int(i64),
  Uint(u64),
  Float(f64),
  String(FixedStr),
  Bytes(Bytes),
  List(OptionList),
  Message(OptionMessage),
  Enum(FixedStr),
  Duration(Duration),
  Timestamp(Timestamp),
}

#[cfg(feature = "serde")]
use serde_json::Value as JsonValue;

#[cfg(feature = "serde")]
impl TryFrom<JsonValue> for OptionValue {
  type Error = String;

  fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
    match value {
      JsonValue::Null => Err("OptionValue cannot be Null".to_string()),

      JsonValue::Bool(b) => Ok(Self::Bool(b)),

      JsonValue::Number(n) => {
        if let Some(i) = n.as_i64() {
          Ok(Self::Int(i))
        } else if let Some(u) = n.as_u64() {
          Ok(Self::Uint(u))
        } else if let Some(f) = n.as_f64() {
          Ok(Self::Float(f))
        } else {
          Err(format!("Number {n} is not representable"))
        }
      }

      JsonValue::String(s) => Ok(Self::String(s.into())),

      JsonValue::Array(arr) => {
        let items: Result<Vec<Self>, _> = arr.into_iter().map(Self::try_from).collect();

        Ok(Self::List(items.into_iter().collect()))
      }

      JsonValue::Object(map) => {
        let mut builder = OptionMessageBuilder::new();

        for (name, val) in map {
          builder.set(name, Self::try_from(val)?);
        }

        Ok(Self::Message(builder.into()))
      }
    }
  }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct OptionMessage {
  inner: Arc<[ProtoOption]>,
}

#[cfg(feature = "serde")]
impl serde::Serialize for OptionMessage {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    let mut map = serializer.serialize_map(Some(self.inner.len()))?;

    for option in self.inner.iter() {
      serde::ser::SerializeMap::serialize_entry(&mut map, &option.name, &option.value)?;
    }

    serde::ser::SerializeMap::end(map)
  }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for OptionMessage {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    struct OptionMessageVisitor;

    impl<'de> serde::de::Visitor<'de> for OptionMessageVisitor {
      type Value = OptionMessage;

      fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("a map of options")
      }

      fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
      where
        M: serde::de::MapAccess<'de>,
      {
        let mut options = Vec::with_capacity(access.size_hint().unwrap_or(0));

        while let Some((key, value)) = access.next_entry::<FixedStr, OptionValue>()? {
          options.push(ProtoOption { name: key, value });
        }

        Ok(OptionMessage {
          inner: options.into(),
        })
      }
    }

    deserializer.deserialize_map(OptionMessageVisitor)
  }
}

impl<'a> IntoIterator for &'a OptionMessage {
  type Item = &'a ProtoOption;
  type IntoIter = core::slice::Iter<'a, ProtoOption>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.inner.iter()
  }
}

impl<T> FromIterator<T> for OptionMessage
where
  T: Into<ProtoOption>,
{
  fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
    let items: Vec<ProtoOption> = iter.into_iter().map(|v| v.into()).collect();

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
      .find_map(|opt| (opt.name.as_ref() == name).then_some(&opt.value))
  }

  #[inline]
  pub fn iter(&self) -> core::slice::Iter<'_, ProtoOption> {
    self.inner.iter()
  }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct OptionMessageBuilder {
  inner: Vec<ProtoOption>,
}

impl OptionMessageBuilder {
  #[must_use]
  pub const fn is_empty(&self) -> bool {
    self.inner.is_empty()
  }

  pub(crate) fn maybe_set(
    &mut self,
    name: impl Into<FixedStr>,
    value: Option<impl Into<OptionValue>>,
  ) -> &mut Self {
    if let Some(value) = value {
      self.set(name.into(), value);
    }
    self
  }

  pub(crate) fn set_boolean(&mut self, name: impl Into<FixedStr>, boolean: bool) -> &mut Self {
    if boolean {
      self.set(name.into(), OptionValue::Bool(true));
    }
    self
  }

  pub(crate) fn add_cel_options(&mut self, rules: Vec<CelProgram>) -> &mut Self {
    if !rules.is_empty() {
      let rule_options: Vec<OptionValue> = rules
        .into_iter()
        .map(|program| program.rule.into())
        .collect();
      self.set("cel", OptionValue::List(rule_options.into()));
    }
    self
  }

  #[inline]
  #[must_use]
  pub fn new() -> Self {
    Self::default()
  }

  #[inline]
  pub fn set(&mut self, name: impl Into<FixedStr>, value: impl Into<OptionValue>) -> &mut Self {
    self.inner.push(ProtoOption {
      name: name.into(),
      value: value.into(),
    });
    self
  }

  #[inline]
  pub fn set_from_option(&mut self, option: impl Into<ProtoOption>) -> &mut Self {
    self.inner.push(option.into());
    self
  }

  pub(crate) fn set_required(&mut self, required: bool) -> &mut Self {
    if required {
      self.set("required", OptionValue::Bool(true));
    }
    self
  }

  pub(crate) fn set_ignore(&mut self, ignore: Ignore) -> &mut Self {
    if !matches!(ignore, Ignore::Unspecified) {
      // The long conversion is necessary here to avoid issues with the i32 representation
      self.set("ignore", <Ignore as Into<OptionValue>>::into(ignore));
    }
    self
  }

  #[inline]
  pub fn iter(&self) -> core::slice::Iter<'_, ProtoOption> {
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

impl<N, V> From<(N, V)> for ProtoOption
where
  N: Into<FixedStr>,
  V: Into<OptionValue>,
{
  fn from(value: (N, V)) -> Self {
    let (name, val) = value;
    Self {
      name: name.into(),
      value: val.into(),
    }
  }
}

impl From<Arc<[ProtoOption]>> for OptionMessage {
  fn from(value: Arc<[ProtoOption]>) -> Self {
    Self { inner: value }
  }
}

impl From<ProtoOption> for OptionMessage {
  fn from(value: ProtoOption) -> Self {
    core::iter::once(value).collect()
  }
}

impl<T> From<Vec<T>> for OptionMessage
where
  T: Into<ProtoOption>,
{
  fn from(value: Vec<T>) -> Self {
    value.into_iter().map(|v| v.into()).collect()
  }
}

impl<N, V> From<Vec<(N, V)>> for OptionValue
where
  N: Into<FixedStr>,
  V: Into<Self>,
{
  fn from(value: Vec<(N, V)>) -> Self {
    Self::Message(value.into())
  }
}

impl From<Vec<ProtoOption>> for OptionValue {
  fn from(value: Vec<ProtoOption>) -> Self {
    Self::Message(value.into())
  }
}

impl<N, V> From<HashMap<N, V>> for OptionMessage
where
  N: Into<FixedStr>,
  V: Into<OptionValue>,
{
  fn from(value: HashMap<N, V>) -> Self {
    value
      .into_iter()
      .map(|(n, v)| ProtoOption {
        name: n.into(),
        value: v.into(),
      })
      .collect()
  }
}

impl<N, V> From<HashMap<N, V>> for OptionValue
where
  N: Into<FixedStr>,
  V: Into<Self>,
{
  fn from(value: HashMap<N, V>) -> Self {
    Self::Message(value.into())
  }
}

impl<N, V> From<BTreeMap<N, V>> for OptionMessage
where
  N: Into<FixedStr>,
  V: Into<OptionValue>,
{
  fn from(value: BTreeMap<N, V>) -> Self {
    value
      .into_iter()
      .map(|(n, v)| ProtoOption {
        name: n.into(),
        value: v.into(),
      })
      .collect()
  }
}

impl<N, V> From<BTreeMap<N, V>> for OptionValue
where
  N: Into<FixedStr>,
  V: Into<Self>,
{
  fn from(value: BTreeMap<N, V>) -> Self {
    Self::Message(value.into())
  }
}

impl<T, const S: usize> From<[T; S]> for OptionMessage
where
  T: Into<ProtoOption>,
{
  fn from(value: [T; S]) -> Self {
    value.into_iter().map(|v| v.into()).collect()
  }
}

impl<const S: usize> From<[ProtoOption; S]> for OptionValue {
  fn from(value: [ProtoOption; S]) -> Self {
    Self::Message(value.into())
  }
}

impl From<OptionMessageBuilder> for OptionMessage {
  fn from(value: OptionMessageBuilder) -> Self {
    value.build()
  }
}

impl IntoIterator for OptionMessageBuilder {
  type Item = ProtoOption;
  type IntoIter = alloc::vec::IntoIter<ProtoOption>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.inner.into_iter()
  }
}

impl<'a> IntoIterator for &'a OptionMessageBuilder {
  type Item = &'a ProtoOption;
  type IntoIter = core::slice::Iter<'a, ProtoOption>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.inner.iter()
  }
}

impl<T> FromIterator<T> for OptionMessageBuilder
where
  T: Into<ProtoOption>,
{
  fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
    let items: Vec<ProtoOption> = iter.into_iter().map(|v| v.into()).collect();

    Self { inner: items }
  }
}

impl<T> Extend<T> for OptionMessageBuilder
where
  T: Into<ProtoOption>,
{
  fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
    self
      .inner
      .extend(iter.into_iter().map(|v| v.into()))
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
  pub fn new_message<I>(items: I) -> Self
  where
    I: Into<OptionMessage>,
  {
    Self::Message(items.into())
  }

  /// Creates a new list option value.
  pub fn new_list<I>(items: I) -> Self
  where
    I: Into<OptionList>,
  {
    Self::List(items.into())
  }

  pub fn new_bytes(bytes: impl IntoBytes) -> Self {
    Self::Bytes(bytes.into_bytes())
  }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OptionList {
  inner: Arc<[OptionValue]>,
}

impl Deref for OptionList {
  type Target = [OptionValue];

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

impl OptionList {
  pub fn iter(&self) -> core::slice::Iter<'_, OptionValue> {
    self.inner.iter()
  }
}

impl<T> FromIterator<T> for OptionList
where
  T: Into<OptionValue>,
{
  fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
    let vec: Vec<OptionValue> = iter.into_iter().map(|i| i.into()).collect();

    Self { inner: vec.into() }
  }
}

impl<'a> IntoIterator for &'a OptionList {
  type Item = &'a OptionValue;
  type IntoIter = core::slice::Iter<'a, OptionValue>;

  fn into_iter(self) -> Self::IntoIter {
    self.inner.iter()
  }
}

impl<T: Into<OptionValue>> From<Vec<T>> for OptionList {
  fn from(value: Vec<T>) -> Self {
    value.into_iter().collect()
  }
}

impl<T: Into<OptionValue> + Ord + Clone> From<SortedList<T>> for OptionList {
  fn from(value: SortedList<T>) -> Self {
    let inner: Vec<OptionValue> = value.iter().cloned().map(|v| v.into()).collect();
    Self {
      inner: inner.into(),
    }
  }
}

impl<T: Into<Self> + Ord + Clone> From<SortedList<T>> for OptionValue {
  fn from(value: SortedList<T>) -> Self {
    Self::List(value.into())
  }
}

impl<T: Into<OptionValue>> From<BTreeSet<T>> for OptionList {
  fn from(value: BTreeSet<T>) -> Self {
    value.into_iter().collect()
  }
}

impl<T: Into<Self>> From<BTreeSet<T>> for OptionValue {
  fn from(value: BTreeSet<T>) -> Self {
    Self::List(value.into())
  }
}

impl<T: Into<OptionValue>> From<HashSet<T>> for OptionList {
  fn from(value: HashSet<T>) -> Self {
    value.into_iter().collect()
  }
}

impl<T: Into<Self>> From<HashSet<T>> for OptionValue {
  fn from(value: HashSet<T>) -> Self {
    Self::List(value.into())
  }
}

impl<T: Into<OptionValue>, const S: usize> From<[T; S]> for OptionList {
  fn from(value: [T; S]) -> Self {
    value.into_iter().collect()
  }
}

impl<T: Into<Self>, const S: usize> From<[T; S]> for OptionValue {
  fn from(value: [T; S]) -> Self {
    Self::List(value.into())
  }
}

impl From<Arc<[OptionValue]>> for OptionList {
  fn from(value: Arc<[OptionValue]>) -> Self {
    Self { inner: value }
  }
}

impl<T: Into<Self>> From<Vec<T>> for OptionValue {
  fn from(value: Vec<T>) -> Self {
    Self::List(value.into())
  }
}

impl From<&'static str> for OptionValue {
  fn from(value: &'static str) -> Self {
    Self::String(value.into())
  }
}

impl From<core::time::Duration> for OptionValue {
  fn from(value: core::time::Duration) -> Self {
    let seconds = value.as_secs().cast_signed();
    #[allow(clippy::cast_possible_truncation)]
    let nanos = value.as_nanos() as i32;

    let duration = Duration { seconds, nanos };

    Self::Duration(duration)
  }
}

impl From<Ignore> for OptionValue {
  fn from(value: Ignore) -> Self {
    let name = match value {
      Ignore::Unspecified => "IGNORE_UNSPECIFIED",
      Ignore::IfZeroValue => "IGNORE_IF_ZERO_VALUE",
      Ignore::Always => "IGNORE_ALWAYS",
    };

    Self::Enum(name.into())
  }
}

impl From<&'static [u8]> for OptionValue {
  fn from(value: &'static [u8]) -> Self {
    Self::Bytes(Bytes::from_static(value))
  }
}

impl<'a, const S: usize> From<&'a [u8; S]> for OptionValue {
  fn from(value: &'a [u8; S]) -> Self {
    Self::Bytes(value.to_vec().into())
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

option_value_conversion!(bool, Bool);
option_value_conversion!(Duration, Duration);
option_value_conversion!(Timestamp, Timestamp);
option_value_conversion!(i64, Int);
option_value_conversion!(i32, Int, as i64);
option_value_conversion!(u64, Uint);
option_value_conversion!(u32, Uint, as u64);
option_value_conversion!(usize, Uint, as u64);
option_value_conversion!(f64, Float);
option_value_conversion!(f32, Float, as f64);
option_value_conversion!(Bytes, Bytes);
option_value_conversion!(OptionMessage, Message);
option_value_conversion!(OptionList, List);
option_value_conversion!(FixedStr, String);
