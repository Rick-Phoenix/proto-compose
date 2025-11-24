use crate::*;

pub(crate) fn process_enum_derive(tokens: DeriveInput) -> Result<TokenStream2, Error> {
  let DeriveInput {
    attrs,
    ident: enum_name,
    data,
    ..
  } = tokens;

  let EnumAttrs {
    reserved_names,
    reserved_numbers,
    options,
    name: proto_name,
    file,
    package,
    full_name,
  } = process_derive_enum_attrs(&enum_name, &attrs).unwrap();

  let data = if let Data::Enum(enum_data) = data {
    enum_data
  } else {
    panic!("The enum derive can only be used on enums");
  };

  let mut output_tokens = TokenStream2::new();

  let mut variants_tokens: Vec<TokenStream2> = Vec::new();

  for variant in data.variants {
    if !variant.fields.is_empty() {
      panic!("Must be a unit variant");
    }

    let EnumVariantAttrs { tag, options, name } =
      process_derive_enum_variants_attrs(&proto_name, &variant.ident, &variant.attrs)?;

    if reserved_numbers.contains(tag) {
      return Err(spanned_error!(
        variant,
        format!("Tag number {tag} is reserved")
      ));
    }

    variants_tokens.push(quote! {
      EnumVariant { name: #name.to_string(), options: #options, tag: #tag, }
    });
  }

  output_tokens.extend(quote! {
    impl ProtoEnumTrait for #enum_name {}

    impl ProtoValidator<#enum_name> for ValidatorMap {
      type Builder = EnumValidatorBuilder;

      fn builder() -> Self::Builder {
        EnumValidator::builder()
      }
    }

    impl AsProtoType for #enum_name {
      fn proto_type() -> ProtoType {
        ProtoType::Single(TypeInfo {
          name: #full_name,
          path: Some(ProtoPath {
            file: #file.into(),
            package: #package.into()
          })
        })
      }
    }

    impl #enum_name {
      #[track_caller]
      pub fn to_enum() -> ProtoEnum {
        ProtoEnum {
          name: #proto_name.into(),
          full_name: #full_name,
          package: #package.into(),
          file: #file.into(),
          variants: vec! [ #(#variants_tokens,)* ],
          reserved_names: #reserved_names,
          reserved_numbers: vec![ #reserved_numbers ],
          options: #options,
        }
      }
    }
  });

  Ok(output_tokens)
}
