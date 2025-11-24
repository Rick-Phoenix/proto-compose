use crate::*;

#[derive(Debug)]
pub struct OneofVariant {
  pub tokens: Variant,
  pub tag: Option<i32>,
  pub name: String,
  pub type_: Path,
}

impl OneofVariant {
  pub fn inject_attr(&mut self, attr: Attribute) {
    self.tokens.attrs.push(attr);
  }
}

#[derive(Debug)]
pub struct OneofData {
  pub data: OneofAttrs,
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
  let oneof_attrs = process_oneof_attrs(&item.ident, &item.attrs, true);

  let mut variants_data: Vec<OneofVariant> = Vec::new();
  let mut used_tags: Vec<i32> = Vec::new();

  for variant in item.variants {
    let ModuleFieldAttrs { tag, name, .. } =
      if let Some(data) = process_module_field_attrs(&variant.ident, &variant.attrs)? {
        data
      } else {
        continue;
      };

    if let Some(tag) = tag {
      used_tags.push(tag);
    }

    let variant_type = if let Fields::Unnamed(variant_fields) = &variant.fields {
      if variant_fields.unnamed.len() != 1 {
        return Err(spanned_error!(
          &variant.ident,
          "Oneof variants must contain a single value"
        ));
      }

      extract_type(&variant_fields.unnamed.first().unwrap().ty)?
    } else {
      return Err(spanned_error!(
        &variant.ident,
        "Oneof variants can only contain unnamed fields"
      ));
    };

    if variant_type.is_option() {
      return Err(spanned_error!(
        variant_type.path(),
        "Oneof variants cannot be Option"
      ));
    }

    variants_data.push(OneofVariant {
      type_: variant_type.path().clone(),
      tokens: variant,
      tag,
      name,
    });
  }

  Ok(OneofData {
    data: oneof_attrs,
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
