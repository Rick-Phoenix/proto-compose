use crate::*;

pub trait ProtoEnum: TryFrom<i32> + Copy + Default + Into<i32> + Send + Sync {
  fn proto_name() -> &'static str;

  fn as_int(&self) -> i32 {
    (*self).into()
  }
}

pub trait ProtoEnumSchema: TryFrom<i32> + Default + ProtoEnum {
  fn proto_path() -> ProtoPath;
  fn proto_schema() -> Enum;

  fn as_proto_name(&self) -> &'static str;
  fn from_proto_name(name: &str) -> Option<Self>;

  #[inline]
  #[must_use]
  fn is_valid(int: i32) -> bool {
    Self::try_from(int).is_ok()
  }

  #[inline]
  #[must_use]
  fn from_int_or_default(int: i32) -> Self {
    int.try_into().unwrap_or_default()
  }
}

#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Template))]
#[cfg_attr(feature = "std", template(path = "enum.proto.j2"))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Enum {
  pub short_name: FixedStr,
  pub name: FixedStr,
  pub package: FixedStr,
  pub file: FixedStr,
  pub variants: Vec<EnumVariant>,
  pub reserved_numbers: Vec<Range<i32>>,
  pub reserved_names: Vec<FixedStr>,
  pub options: Vec<ProtoOption>,
  // Not a static str because we compose this
  // by default with module_path!() + ident
  pub rust_path: String,
}

#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EnumVariant {
  pub name: FixedStr,
  pub tag: i32,
  pub options: Vec<ProtoOption>,
}

impl Enum {
  pub(crate) fn render_reserved_names(&self) -> Option<String> {
    render_reserved_names(&self.reserved_names)
  }

  pub(crate) fn render_reserved_numbers(&self) -> Option<String> {
    render_reserved_numbers(&self.reserved_numbers)
  }
}
