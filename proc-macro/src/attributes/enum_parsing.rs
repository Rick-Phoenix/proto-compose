use crate::*;

pub struct EnumData {
  pub name: String,
  pub tokens: ItemEnum,
}

impl From<EnumData> for ItemEnum {
  fn from(value: EnumData) -> Self {
    value.tokens
  }
}

pub fn parse_enum(item: ItemEnum) -> Result<EnumData, Error> {
  let ModuleEnumAttrs { name } = process_module_enum_attrs(&item.ident, &item.attrs)?;

  Ok(EnumData { name, tokens: item })
}
