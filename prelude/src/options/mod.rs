use std::sync::Arc;

use askama::Template;
use proto_types::{Duration, Timestamp};

#[derive(Clone, Debug, PartialEq)]
pub struct ProtoOption {
  pub name: Arc<str>,
  pub value: OptionValue,
}

impl ProtoOption {
  pub(crate) fn render_as_field_option(&self) -> String {
    let Self { name, value } = self;

    format!("{name} = {value}")
  }

  pub(crate) fn render(&self) -> String {
    let Self { name, value } = self;

    format!("option {name} = {value};")
  }
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
  List(Arc<[OptionValue]>),
  Message(Arc<[(Arc<str>, OptionValue)]>),
  Enum(Arc<str>),
  Duration(Duration),
  Timestamp(Timestamp),
}

impl OptionValue {
  pub(crate) fn is_short(&self) -> bool {
    match self {
      Self::List(list) => list.len() <= 5 && list.iter().all(OptionValue::is_short),
      Self::String(str) => str.chars().count() <= 5,
      Self::Duration(_) | Self::Timestamp(_) | Self::Message(_) => false,
      _ => true,
    }
  }

  /// Creates a new message option value.
  pub fn new_message<S, V, I>(items: I) -> Self
  where
    S: Into<Arc<str>>,
    V: Into<OptionValue>,
    I: IntoIterator<Item = (S, V)>,
  {
    let items: Vec<(Arc<str>, OptionValue)> = items
      .into_iter()
      .map(|(name, val)| (name.into(), val.into()))
      .collect();

    Self::Message(items.into())
  }

  /// Creates a new list option value.
  pub fn new_list<I, V>(items: I) -> Self
  where
    V: Into<OptionValue>,
    I: IntoIterator<Item = V>,
  {
    let items: Vec<OptionValue> = items.into_iter().map(|v| v.into()).collect();

    Self::List(items.into())
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

impl<T: Into<OptionValue> + Clone> From<Arc<[T]>> for OptionValue {
  fn from(value: Arc<[T]>) -> Self {
    OptionValue::List(
      value
        .iter()
        .map(|item| (*item).clone().into())
        .collect::<Vec<OptionValue>>()
        .into(),
    )
  }
}

impl<T: Into<OptionValue> + Clone> From<Vec<T>> for OptionValue {
  fn from(value: Vec<T>) -> Self {
    OptionValue::List(
      value
        .into_iter()
        .map(|item| item.clone().into())
        .collect::<Vec<OptionValue>>()
        .into(),
    )
  }
}

impl<T: Into<OptionValue> + Clone> From<&[T]> for OptionValue {
  fn from(value: &[T]) -> Self {
    OptionValue::List(
      value
        .iter()
        .map(|item| item.clone().into())
        .collect::<Vec<OptionValue>>()
        .into(),
    )
  }
}

impl From<&str> for OptionValue {
  fn from(value: &str) -> Self {
    OptionValue::String(value.into())
  }
}

impl From<Arc<str>> for OptionValue {
  fn from(value: Arc<str>) -> Self {
    OptionValue::String(value)
  }
}

impl From<std::time::Duration> for OptionValue {
  fn from(value: std::time::Duration) -> Self {
    let seconds = value.as_secs() as i64;
    let nanos = value.as_nanos() as i32;

    let duration = Duration { seconds, nanos };

    OptionValue::Duration(duration)
  }
}

option_value_conversion!(Arc<[(Arc<str>, OptionValue)]>, Message);
option_value_conversion!(bool, Bool);
option_value_conversion!(Duration, Duration);
option_value_conversion!(Timestamp, Timestamp);
option_value_conversion!(i64, Int);
option_value_conversion!(i32, Int, as i64);
option_value_conversion!(u64, Uint);
option_value_conversion!(u32, Uint, as u64);
option_value_conversion!(f64, Float);
option_value_conversion!(f32, Float, as f64);
