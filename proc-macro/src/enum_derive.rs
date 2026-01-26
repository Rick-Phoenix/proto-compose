use crate::*;

// This cannot be reused from the proc macro because that one gets
// the parent message name with a method, this one gets it from an attribute
// therefore we cannot pass it from the proc macro to this one,
// only via reflection
pub fn named_enum_derive(item: &ItemEnum) -> TokenStream2 {
  let mut name: Option<String> = None;

  for attr in &item.attrs {
    if attr.path().is_ident("proto") {
      let _ = attr.parse_nested_meta(|meta| {
        let ident_str = meta.ident_str()?;

        match ident_str.as_str() {
          "name" => {
            name = Some(meta.parse_value::<LitStr>()?.value());
          }
          _ => drain_token_stream!(meta.input),
        };

        Ok(())
      });
    }
  }

  let ident = &item.ident;
  let name = name.unwrap_or_else(|| to_pascal_case(&item.ident.to_string()));

  quote! {
    impl ::prelude::ProtoEnum for #ident {
      #[inline]
      fn proto_name() -> &'static str {
        #name
      }
    }

    impl ::prelude::ProtoValidation for #ident {
      #[doc(hidden)]
      type Target = i32;
      #[doc(hidden)]
      type Stored = i32;
      #[doc(hidden)]
      type Validator = ::prelude::EnumValidator<#ident>;
      #[doc(hidden)]
      type Builder = ::prelude::EnumValidatorBuilder<#ident>;

      type UniqueStore<'a>
        = ::prelude::CopyHybridStore<i32>
        where
          Self: 'a;
    }
  }
}
