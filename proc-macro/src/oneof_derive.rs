use crate::*;

pub(crate) fn process_oneof_derive(item: &mut ItemEnum) -> Result<TokenStream2, Error> {
  let ItemEnum {
    attrs,
    ident: enum_name,
    variants,
    ..
  } = item;

  let OneofAttrs {
    options,
    name: proto_name,
    required,
  } = process_oneof_attrs(&enum_name, attrs, false);

  let mut variants_tokens: Vec<TokenStream2> = Vec::new();

  for variant in variants {
    let field_attrs =
      if let Some(attrs) = process_derive_field_attrs(&variant.ident, &variant.attrs)? {
        attrs
      } else {
        continue;
      };

    let FieldAttrs {
      tag,
      validator,
      options,
      name,
      ..
    } = field_attrs;

    let proto_type = if let Fields::Unnamed(variant_fields) = &variant.fields {
      if variant_fields.unnamed.len() != 1 {
        panic!("Oneof variants must contain a single value");
      }

      match variant_fields.unnamed.first().unwrap().ty.clone() {
        Type::Path(type_path) => type_path.path,

        _ => panic!("Must be a path type"),
      }
    } else {
      panic!("Enum can only have one unnamed field")
    };

    let validator_tokens = if let Some(validator) = validator {
      match validator {
        ValidatorExpr::Call(call) => {
          quote! { Some(<ValidatorMap as ProtoValidator<#proto_type>>::from_builder(#call)) }
        }
        ValidatorExpr::Closure(closure) => {
          quote! { Some(<ValidatorMap as ProtoValidator<#proto_type>>::build_rules(#closure)) }
        }
      }
    } else {
      quote! { None }
    };

    variants_tokens.push(quote! {
      ProtoField {
        name: #name.to_string(),
        options: #options,
        type_: <#proto_type as AsProtoType>::proto_type(),
        validator: #validator_tokens,
        tag: #tag,
      }
    });
  }

  let required_option_tokens = required.then(|| quote! { options.push(oneof_required()); });

  let output_tokens = quote! {
    impl ProtoOneof for #enum_name {
      fn fields() -> Vec<ProtoField> {
        vec![ #(#variants_tokens,)* ]
      }
    }

    impl #enum_name {
      #[track_caller]
      pub fn to_oneof() -> Oneof {
        let mut options: Vec<ProtoOption> = #options;

        #required_option_tokens

        Oneof {
          name: #proto_name.into(),
          fields: Self::fields(),
          options,
        }
      }
    }
  };

  Ok(output_tokens)
}
