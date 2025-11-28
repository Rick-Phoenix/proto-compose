use quote::format_ident;

use crate::*;

fn create_shadow_struct(item: &ItemStruct) -> ItemStruct {
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

pub(crate) fn process_message_derive_shadow(
  item: &mut ItemStruct,
  message_attrs: MessageAttrs,
) -> Result<TokenStream2, Error> {
  let mut output_tokens = TokenStream2::new();

  let mut shadow_struct = create_shadow_struct(item);

  let ItemStruct {
    ident: orig_struct_name,
    fields,
    ..
  } = item;

  let mut proto_fields_data: Vec<TokenStream2> = Vec::new();

  let mut from_proto = TokenStream2::new();
  let mut into_proto = TokenStream2::new();

  let orig_struct_fields = fields.iter_mut();
  let shadow_struct_fields = shadow_struct.fields.iter_mut();

  let mut ignored_fields: Vec<Ident> = Vec::new();

  for (src_field, dst_field) in orig_struct_fields.zip(shadow_struct_fields) {
    let src_field_ident = src_field.ident.as_ref().expect("Expected named field");

    let field_attrs = process_derive_field_attrs(src_field_ident, &src_field.attrs)?;

    let field_attrs_from_proto = field_attrs.from_proto.clone();
    let field_attrs_into_proto = field_attrs.into_proto.clone();
    let is_enum = field_attrs.kind.is_enum();

    let src_field_type = TypeInfo::from_type(&src_field.ty, field_attrs.kind.clone())?;

    if field_attrs.is_ignored {
      ignored_fields.push(src_field.ident.clone().unwrap());
    } else {
      let field_tokens = process_field(
        &mut FieldOrVariant::Field(dst_field),
        field_attrs.clone(),
        &src_field_type,
        OutputType::Change,
      )?;

      proto_fields_data.push(field_tokens);

      if message_attrs.into_proto.is_none() {
        let conversion_call = if let Some(expr) = field_attrs_into_proto {
          match expr {
            PathOrClosure::Path(path) => quote! { #path(value.#src_field_ident) },
            PathOrClosure::Closure(closure) => {
              quote! {
                prelude::apply(value.#src_field_ident, #closure)
              }
            }
          }
        } else {
          let call = src_field_type.rust_type.into_proto();

          quote! { value.#src_field_ident.#call }
        };

        into_proto.extend(quote! {
          #src_field_ident: #conversion_call,
        });
      }
    }

    if message_attrs.from_proto.is_none() {
      let conversion_call = if let Some(expr) = field_attrs_from_proto {
        match expr {
          PathOrClosure::Path(path) => quote! { #path(value.#src_field_ident) },
          PathOrClosure::Closure(closure) => {
            quote! {
              prelude::apply(value.#src_field_ident, #closure)
            }
          }
        }
      } else if field_attrs.is_ignored {
        quote! { Default::default() }
      } else {
        let call = src_field_type.from_proto();

        quote! { value.#src_field_ident.#call }
      };

      from_proto.extend(quote! {
        #src_field_ident: #conversion_call,
      });
    }
  }

  if let Fields::Named(fields) = &mut shadow_struct.fields {
    let old_fields = std::mem::take(&mut fields.named);

    fields.named = old_fields
      .into_iter()
      .filter(|f| !ignored_fields.contains(f.ident.as_ref().unwrap()))
      .collect();
  }

  let schema_impls = message_schema_impls(orig_struct_name, &message_attrs, proto_fields_data);

  output_tokens.extend(schema_impls);

  let shadow_struct_ident = &shadow_struct.ident;

  output_tokens.extend(quote! {
    #[derive(prost::Message, Clone, PartialEq)]
    #shadow_struct

    impl AsProtoType for #shadow_struct_ident {
      fn proto_type() -> ProtoType {
        <#orig_struct_name as AsProtoType>::proto_type()
      }
    }
  });

  let from_proto = if let Some(expr) = &message_attrs.from_proto {
    match expr {
      PathOrClosure::Path(path) => quote! { #path(value) },
      PathOrClosure::Closure(closure) => quote! {
        prelude::apply(value, #closure)
      },
    }
  } else {
    quote! {
      Self {
        #from_proto
      }
    }
  };

  let from_proto_impl = quote! {
    impl From<#shadow_struct_ident> for #orig_struct_name {
      #[allow(clippy::redundant_closure)]
      fn from(value: #shadow_struct_ident) -> Self {
        #from_proto
      }
    }

    impl #orig_struct_name {
      pub fn from_proto(value: #shadow_struct_ident) -> Self {
        value.into()
      }

      pub fn into_proto(self) -> #shadow_struct_ident {
        self.into()
      }
    }
  };

  output_tokens.extend(from_proto_impl);

  let into_proto = if let Some(expr) = &message_attrs.into_proto {
    match expr {
      PathOrClosure::Path(path) => quote! { #path(value) },
      PathOrClosure::Closure(closure) => quote! {
        prelude::apply(value, #closure)
      },
    }
  } else {
    quote! {
      Self {
        #into_proto
      }
    }
  };

  let into_proto_impl = quote! {
    impl From<#orig_struct_name> for #shadow_struct_ident {
      #[allow(clippy::redundant_closure)]
      fn from(value: #orig_struct_name) -> Self {
        #into_proto
      }
    }
  };

  output_tokens.extend(into_proto_impl);

  Ok(output_tokens)
}

pub(crate) fn process_message_derive(item: &mut ItemStruct) -> Result<TokenStream2, Error> {
  let message_attrs = process_derive_message_attrs(&item.ident, &item.attrs)?;

  if message_attrs.direct {
    process_message_derive_direct(item, message_attrs)
  } else {
    process_message_derive_shadow(item, message_attrs)
  }
}

pub(crate) fn process_message_derive_direct(
  item: &mut ItemStruct,
  message_attrs: MessageAttrs,
) -> Result<TokenStream2, Error> {
  let mut output_tokens = TokenStream2::new();

  let prost_message_attr: Attribute = parse_quote!(#[derive(prost::Message, Clone, PartialEq)]);

  item.attrs.push(prost_message_attr);

  let ItemStruct {
    ident: struct_name,
    fields,
    ..
  } = item;

  let mut fields_data: Vec<TokenStream2> = Vec::new();

  for src_field in fields {
    let src_field_ident = src_field.ident.as_ref().expect("Expected named field");

    let field_attrs = process_derive_field_attrs(src_field_ident, &src_field.attrs)?;

    if field_attrs.is_ignored {
      return Err(spanned_error!(
        src_field,
        "Fields cannot be ignored in a direct impl"
      ));
    }

    let type_info = TypeInfo::from_type(&src_field.ty, field_attrs.kind.clone())?;

    let field_tokens = process_field(
      &mut FieldOrVariant::Field(src_field),
      field_attrs,
      &type_info,
      OutputType::Keep,
    )?;

    fields_data.push(field_tokens);
  }

  let schema_impls = message_schema_impls(struct_name, &message_attrs, fields_data);

  output_tokens.extend(schema_impls);

  Ok(output_tokens)
}
