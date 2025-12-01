use crate::*;

pub struct ProstAttrs<'a> {
  pub proto_type: &'a ProtoType,
  pub cardinality: ProstCardinality,
  pub tag: i32,
}

impl<'a> ProstAttrs<'a> {
  pub fn from_type_info(type_info: &'a TypeInfo, tag: i32) -> Self {
    let cardinality = match &type_info.rust_type {
      RustType::Option(_) => ProstCardinality::Optional,
      RustType::BoxedMsg(_) => ProstCardinality::Boxed,
      RustType::Vec(_) => ProstCardinality::Repeated,

      _ => ProstCardinality::Single,
    };

    Self {
      proto_type: &type_info.proto_type,
      cardinality,
      tag,
    }
  }
}

impl<'a> ToTokens for ProstAttrs<'a> {
  fn to_tokens(&self, tokens: &mut TokenStream2) {
    let Self {
      proto_type,
      cardinality,
      tag,
    } = self;

    let tag_as_str = tag.to_string();

    let type_attr = proto_type.as_prost_attr_type();

    let output = quote! { #[prost(#type_attr, #cardinality tag = #tag_as_str)] };

    tokens.extend(output);
  }
}

pub enum ProstCardinality {
  Repeated,
  Optional,
  Single,
  Boxed,
}

impl ToTokens for ProstCardinality {
  fn to_tokens(&self, tokens: &mut TokenStream2) {
    let output = match self {
      ProstCardinality::Repeated => quote! { repeated, },
      ProstCardinality::Optional => quote! { optional, },
      ProstCardinality::Single => TokenStream2::new(),
      ProstCardinality::Boxed => quote! { optional, boxed, },
    };

    tokens.extend(output);
  }
}
