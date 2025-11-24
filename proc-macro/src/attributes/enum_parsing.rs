use crate::*;

pub struct EnumData {
  pub name: String,
  pub reserved_numbers: ReservedNumbers,
  pub variants: Vec<EnumVariant>,
  pub used_tags: Vec<i32>,
  pub tokens: EnumRaw,
}

impl EnumData {
  pub fn inject_attr(&mut self, attr: Attribute) {
    self.tokens.attrs.push(attr);
  }
}

impl From<EnumData> for ItemEnum {
  fn from(value: EnumData) -> Self {
    let variants: Punctuated<Variant, Token![,]> = value
      .variants
      .into_iter()
      .map(|variant| variant.tokens)
      .collect();

    Self {
      attrs: value.tokens.attrs,
      vis: value.tokens.vis,
      enum_token: token::Enum::default(),
      ident: value.tokens.ident,
      generics: value.tokens.generics,
      brace_token: token::Brace::default(),
      variants,
    }
  }
}

pub struct EnumVariant {
  pub tokens: Variant,
  pub name: String,
  pub tag: Option<i32>,
}

impl EnumVariant {
  pub fn inject_attr(&mut self, attr: Attribute) {
    self.tokens.attrs.push(attr);
  }
}

pub fn parse_enum(item: ItemEnum) -> Result<EnumData, Error> {
  let ModuleEnumAttrs {
    reserved_numbers,
    name,
  } = process_module_enum_attrs(&item.ident, &item.attrs)?;

  let mut variants_data: Vec<EnumVariant> = Vec::new();
  let mut used_tags: Vec<i32> = Vec::new();

  for variant in item.variants {
    if !matches!(variant.fields, Fields::Unit) {
      return Err(spanned_error!(
        variant,
        "Protobuf enums can only have unit variants"
      ));
    }

    let ModuleEnumVariantAttrs { name, tag, .. } =
      process_module_enum_variants_attrs(&name, &variant.ident, &variant.attrs)?;

    if let Some(tag) = tag {
      used_tags.push(tag);
    }

    variants_data.push(EnumVariant {
      tokens: variant,
      name,
      tag,
    });
  }

  Ok(EnumData {
    name,
    reserved_numbers,
    variants: variants_data,
    tokens: EnumRaw {
      attrs: item.attrs,
      vis: item.vis,
      ident: item.ident,
      generics: item.generics,
    },
    used_tags,
  })
}
