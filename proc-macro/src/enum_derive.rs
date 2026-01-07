use crate::*;

// This cannot be reused from the proc macro because that one gets
// the parent message name with a method, this one gets it from an attribute
// therefore we cannot pass it from the proc macro to this one,
// only via reflection
pub fn named_enum_derive(item: &ItemEnum) -> syn::Result<TokenStream2> {
  let mut name: Option<String> = None;

  for attr in &item.attrs {
    if attr.path().is_ident("proto") {
      attr.parse_nested_meta(|meta| {
        let ident_str = meta.ident_str()?;

        match ident_str.as_str() {
          "name" => {
            name = Some(meta.parse_value::<LitStr>()?.value());
          }
          _ => return Err(meta.error("Unknown attribute")),
        };

        Ok(())
      })?;
    }
  }

  let ident = &item.ident;
  let name = name.unwrap_or_else(|| to_pascal_case(&item.ident.to_string()));

  Ok(quote! {
    impl ::prelude::ProtoEnum for #ident {
      fn proto_name() -> &'static str {
        #name
      }
    }

    impl ::prelude::ProtoValidator for #ident {
      #[doc(hidden)]
      type Target = i32;
      #[doc(hidden)]
      type Validator = ::prelude::EnumValidator<#ident>;
      #[doc(hidden)]
      type Builder = ::prelude::EnumValidatorBuilder<#ident>;
    }
  })
}
