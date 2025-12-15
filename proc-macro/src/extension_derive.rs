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
      ExtendTarget::FileOptions => "FileOptions",
      ExtendTarget::MessageOptions => "MessageOptions",
      ExtendTarget::FieldOptions => "FieldOptions",
      ExtendTarget::OneofOptions => "OneofOptions",
      ExtendTarget::EnumOptions => "EnumOptions",
      ExtendTarget::EnumValueOptions => "EnumValueOptions",
      ExtendTarget::ServiceOptions => "ServiceOptions",
      ExtendTarget::MethodOptions => "MethodOptions",
    };

    tokens.extend(output.to_token_stream());
  }
}

pub fn process_extension_derive(
  args: TokenStream,
  item: &mut ItemStruct,
) -> Result<TokenStream2, Error> {
  let parser = Punctuated::<MetaNameValue, Token![,]>::parse_terminated;
  let args = parser.parse(args)?;

  let ItemStruct { ident, fields, .. } = item;

  let mut target: Option<ExtendTarget> = None;
  let mut fields_tokens: Vec<TokenStream2> = Vec::new();

  for arg in args {
    let ident = arg.path.require_ident()?.to_string();

    match ident.as_str() {
      "target" => {
        let path = arg.value.as_path()?;
        target = Some(ExtendTarget::from_ident(path.require_ident()?)?);
      }
      _ => bail!(arg, "Unknown attribute `{ident}`"),
    };
  }

  let target = target.ok_or(error!(&ident, "Missing target attribute"))?;

  for field in fields {
    let field_ident = field
      .ident
      .as_ref()
      .ok_or(error!(&field, "Expected a named field"))?;

    let rust_type = TypeInfo::from_type(&field.ty)?;

    let field_data = process_derive_field_attrs(field_ident, &rust_type, &field.attrs)?;

    let FieldAttrs {
      tag,
      options,
      name,
      proto_field,
      ..
    } = if let FieldAttrData::Normal(data) = field_data {
      *data
    } else {
      bail!(&field, "Cannot ignore fields in extensions");
    };

    let type_ctx = TypeContext::new(rust_type, &proto_field)?;
    let options_tokens = tokens_or_default!(options, quote! { vec![] });
    let field_type_tokens = type_ctx.proto_field.as_proto_type_trait_expr();

    fields_tokens.push(quote! {
      ::prelude::ProtoField {
        name: #name.to_string(),
        tag: #tag,
        options: #options_tokens,
        type_: #field_type_tokens,
        validator: None,
      }
    });
  }

  item.fields = Fields::Unit;

  Ok(quote! {
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
