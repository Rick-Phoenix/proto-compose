use crate::*;

pub fn create_shadow_struct(item: &ItemStruct) -> ItemStruct {
  let item_fields = if let Fields::Named(fields) = &item.fields {
    fields.named.iter().map(|f| Field {
      attrs: vec![],
      vis: f.vis.clone(),
      mutability: syn::FieldMutability::None,
      ident: f.ident.clone(),
      colon_token: f.colon_token,
      ty: f.ty.clone(),
    })
  } else {
    unreachable!()
  };

  ItemStruct {
    attrs: vec![],
    vis: Visibility::Public(token::Pub::default()),
    struct_token: token::Struct::default(),
    ident: format_ident!("{}Proto", item.ident),
    generics: item.generics.clone(),
    fields: Fields::Named(syn::FieldsNamed {
      brace_token: token::Brace::default(),
      named: item_fields.collect(),
    }),
    semi_token: None,
  }
}

pub fn create_shadow_enum(item: &ItemEnum) -> ItemEnum {
  let variants = item.variants.iter().map(|variant| Variant {
    attrs: vec![],
    ident: variant.ident.clone(),
    discriminant: variant.discriminant.clone(),
    fields: variant.fields.clone(),
  });

  ItemEnum {
    attrs: vec![],
    vis: Visibility::Public(token::Pub::default()),
    enum_token: token::Enum::default(),
    ident: format_ident!("{}Proto", item.ident),
    generics: item.generics.clone(),
    brace_token: token::Brace::default(),
    variants: variants.collect(),
  }
}
