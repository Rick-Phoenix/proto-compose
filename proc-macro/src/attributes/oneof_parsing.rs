use crate::*;

pub struct OneofVariant {
  pub tokens: Variant,
  pub tag: Option<i32>,
  pub name: String,
  pub is_ignored: bool,
}

impl OneofVariant {
  pub fn inject_attr(&mut self, attr: Attribute) {
    self.tokens.attrs.push(attr);
  }
}

pub struct OneofData {
  pub tokens: EnumRaw,
  pub variants: Vec<OneofVariant>,
  pub used_tags: Vec<i32>,
}

impl From<OneofData> for ItemEnum {
  fn from(value: OneofData) -> Self {
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

#[derive(Debug)]
pub struct EnumRaw {
  pub attrs: Vec<Attribute>,
  pub vis: Visibility,
  pub ident: Ident,
  pub generics: Generics,
}

pub fn parse_oneof(item: ItemEnum) -> Result<OneofData, Error> {
  let mut variants_data: Vec<OneofVariant> = Vec::new();
  let mut used_tags: Vec<i32> = Vec::new();

  for variant in item.variants {
    let ModuleFieldAttrs {
      tag,
      name,
      is_ignored,
      ..
    } = process_module_field_attrs(&variant.ident, &variant.attrs)?;

    if let Some(tag) = tag {
      used_tags.push(tag);
    }

    variants_data.push(OneofVariant {
      tokens: variant,
      tag,
      name,
      is_ignored,
    });
  }

  Ok(OneofData {
    tokens: EnumRaw {
      attrs: item.attrs,
      vis: item.vis,
      ident: item.ident,
      generics: item.generics,
    },
    variants: variants_data,
    used_tags,
  })
}
