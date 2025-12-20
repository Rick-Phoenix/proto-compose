mod common_options;

use std::sync::Arc;

use askama::Template;
pub use common_options::*;
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
  List(Arc<[Self]>),
  Message(Arc<[(Arc<str>, Self)]>),
  Enum(Arc<str>),
  Duration(Duration),
  Timestamp(Timestamp),
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
    let items: Vec<(Arc<str>, Self)> = items
      .into_iter()
      .map(|(name, val)| (name.into(), val.into()))
      .collect();

    Self::Message(items.into())
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
