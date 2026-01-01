use crate::*;

#[allow(clippy::enum_variant_names)]
enum ExtendTarget {
  FileOptions,
  MessageOptions,
  FieldOptions,
  OneofOptions,
  EnumOptions,
  EnumValueOptions,
  ServiceOptions,
  MethodOptions,
}

impl ExtendTarget {
  pub fn from_ident(ident: &Ident) -> Result<Self, Error> {
    let ident_str = ident.to_string();

    let output = match ident_str.as_str() {
      "FileOptions" => Self::FileOptions,
      "MessageOptions" => Self::MessageOptions,
      "FieldOptions" => Self::FieldOptions,
      "OneofOptions" => Self::OneofOptions,
      "EnumOptions" => Self::EnumOptions,
      "EnumValueOptions" => Self::EnumValueOptions,
      "ServiceOptions" => Self::ServiceOptions,
      "MethodOptions" => Self::MethodOptions,
      _ => bail!(ident, "Unrecognized extension target"),
    };

    Ok(output)
  }
}

impl ToTokens for ExtendTarget {
  fn to_tokens(&self, tokens: &mut TokenStream2) {
    let output = match self {
      ExtendTarget::FileOptions => "google.protobuf.FileOptions",
      ExtendTarget::MessageOptions => "google.protobuf.MessageOptions",
      ExtendTarget::FieldOptions => "google.protobuf.FieldOptions",
      ExtendTarget::OneofOptions => "google.protobuf.OneofOptions",
      ExtendTarget::EnumOptions => "google.protobuf.EnumOptions",
      ExtendTarget::EnumValueOptions => "google.protobuf.EnumValueOptions",
      ExtendTarget::ServiceOptions => "google.protobuf.ServiceOptions",
      ExtendTarget::MethodOptions => "google.protobuf.MethodOptions",
    };

    tokens.extend(output.to_token_stream());
  }
}

pub fn process_extension_derive(
  args: TokenStream2,
  item: &mut ItemStruct,
) -> Result<TokenStream2, Error> {
  let ItemStruct { ident, fields, .. } = item;

  let args_span = args.span();

  let mut target: Option<ExtendTarget> = None;
  let mut fields_tokens: Vec<TokenStream2> = Vec::new();

  let parser = syn::meta::parser(|meta| {
    let ident = meta.ident_str()?;

    match ident.as_str() {
      "target" => {
        let target_ident = meta.parse_value::<Ident>()?;

        target = Some(ExtendTarget::from_ident(&target_ident)?);
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
      bail!(field, "Tag is missing");
    }

    let field_type_tokens = proto_field.field_proto_type_tokens();

    fields_tokens.push(quote! {
      ::prelude::ProtoField {
        name: #proto_name.to_string(),
        tag: #tag,
        options: #options,
        type_: #field_type_tokens,
        validator: None,
      }
    });
  }

  item.fields = Fields::Unit;

  Ok(quote! {
    ::prelude::inventory::submit! {
      ::prelude::RegistryExtension {
        file: __PROTO_FILE.file,
        package: __PROTO_FILE.package,
        extension: || #ident::as_proto_extension()
      }
    }

    impl #ident {
      pub fn as_proto_extension() -> ::prelude::Extension {
        ::prelude::Extension {
          target: #target,
          fields: vec![ #(#fields_tokens),* ]
        }
      }
    }
  })
}
