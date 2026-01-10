use crate::*;

pub fn process_extension_derive(
  args: TokenStream2,
  item: &mut ItemStruct,
) -> Result<TokenStream2, Error> {
  let ItemStruct { ident, fields, .. } = item;

  let args_span = args.span();

  let mut target: Option<Ident> = None;
  let mut fields_tokens: Vec<TokenStream2> = Vec::new();

  let parser = syn::meta::parser(|meta| {
    let ident = meta.ident_str()?;

    match ident.as_str() {
      "target" => {
        let target_ident = meta.parse_value::<Ident>()?;

        let ident_str = target_ident.to_string();

        let known_targets = [
          "FileOptions",
          "MessageOptions",
          "FieldOptions",
          "OneofOptions",
          "EnumOptions",
          "EnumValueOptions",
          "ServiceOptions",
          "MethodOptions",
        ];

        if !known_targets.contains(&ident_str.as_str()) {
          bail!(
            ident,
            "Unknown protobuf extension target. Allowed values are: {known_targets:?}"
          );
        }

        target = Some(target_ident);
      }
      _ => return Err(meta.error("Unknown attribute")),
    };

    Ok(())
  });

  parser.parse2(args)?;

  let target = target.ok_or_else(|| error_with_span!(args_span, "Missing target attribute"))?;

  for field in fields {
    let ExtensionFieldAttrs {
      tag,
      options,
      proto_name,
      proto_field,
    } = process_extension_field_attrs(field)?;

    if tag.is_none() {
      bail!(
        field,
        "Missing protobuf tag. You can set it with `#[proto(tag = 123)]`"
      );
    }

    let field_type_tokens = proto_field.field_proto_type_tokens();

    fields_tokens.push(quote_spanned! {field.ident.span()=>
      ::prelude::Field {
        name: #proto_name,
        tag: #tag,
        options: #options.into_iter().collect(),
        type_: #field_type_tokens,
        validator: None,
      }
    });
  }

  item.fields = Fields::Unit;

  Ok(quote! {
    impl ::prelude::ProtoExtension for #ident {
      fn as_proto_extension() -> ::prelude::Extension {
        ::prelude::Extension {
          target: ::prelude::ExtensionTarget::#target,
          fields: vec![ #(#fields_tokens),* ]
        }
      }
    }
  })
}
