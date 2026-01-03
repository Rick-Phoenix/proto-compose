use crate::{oneof_schema_impl::fallback_oneof_schema_impl, *};

pub struct FallbackImpls<'a> {
  pub error: Error,
  pub orig_ident: &'a Ident,
  pub shadow_ident: Option<&'a Ident>,
  pub kind: InputItemKind,
}

impl<'a> FallbackImpls<'a> {
  pub fn generate_fallback_impls(&self) -> TokenStream2 {
    let Self {
      error,
      orig_ident,
      kind,
      ..
    } = self;

    let shadow_ident = self.shadow_ident;
    let proto_ident = self.proto_ident();

    let err = error.to_compile_error();

    let validator_impl = match kind {
      InputItemKind::Oneof => fallback_oneof_validator(proto_ident),
      InputItemKind::Message => fallback_message_validator_impl(proto_ident),
    };

    let schema_impl = match kind {
      InputItemKind::Oneof => fallback_oneof_schema_impl(proto_ident),
      InputItemKind::Message => fallback_message_schema_impl(orig_ident, shadow_ident),
    };

    let mut to_wrap = vec![validator_impl, schema_impl];

    if let Some(shadow_ident) = shadow_ident {
      to_wrap.push(fallback_conversion_impls(orig_ident, shadow_ident, *kind));
    }

    let wrapped = wrap_with_imports(to_wrap);

    let fallback_derive_impls = self.fallback_derive_impls();
    let fallback_prost_impls = self.fallback_prost_impls();

    quote! {
      #fallback_derive_impls
      #fallback_prost_impls

      #wrapped
      #err
    }
  }

  fn proto_ident(&self) -> &'a Ident {
    self.shadow_ident.unwrap_or(self.orig_ident)
  }

  fn fallback_prost_impls(&self) -> TokenStream2 {
    let target_ident = self.proto_ident();

    match self.kind {
      InputItemKind::Oneof => {
        quote! {
          impl #target_ident {
            pub fn encode(&self, buf: &mut impl ::prost::bytes::BufMut) {
              unimplemented!()
            }

            pub fn merge(
              _: &mut ::core::option::Option<Self>,
              _: u32,
              _: ::prost::encoding::wire_type::WireType,
              _: &mut impl ::prost::bytes::Buf,
              _: ::prost::encoding::DecodeContext,
            ) -> ::core::result::Result<(), ::prost::DecodeError>
            {
              unimplemented!()
            }

            pub fn encoded_len(&self) -> usize {
              unimplemented!()
            }
          }
        }
      }
      InputItemKind::Message => {
        quote! {
          impl ::prost::Message for #target_ident {
            fn encoded_len(&self) -> usize {
              unimplemented!()
            }

            fn encode_raw(&self, _: &mut impl bytes::buf::BufMut) { unimplemented!() }

            fn merge_field(&mut self, _: u32, _: prost::encoding::WireType, _: &mut impl bytes::buf::Buf, _: prost::encoding::DecodeContext) -> std::result::Result<(), prost::DecodeError> { unimplemented!() }

            fn clear(&mut self) {}
          }
        }
      }
    }
  }

  fn fallback_derive_impls(&self) -> TokenStream2 {
    let target_ident = self.proto_ident();

    let mut output = quote! {
      impl Default for #target_ident {
        fn default() -> Self {
          unimplemented!()
        }
      }

      impl Clone for #target_ident {
        fn clone(&self) -> Self {
          unimplemented!()
        }
      }

      impl std::fmt::Debug for #target_ident {
        fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
          unimplemented!()
        }
      }

      impl PartialEq for #target_ident {
        fn eq(&self, _: &Self) -> bool {
          unimplemented!()
        }
      }
    };

    if cfg!(feature = "cel") {
      output.extend(quote! {
        impl #target_ident {
          pub fn try_into_cel_recursive(self, _: usize) -> Result<::prelude::cel::Value, ::prelude::proto_types::cel::CelConversionError> {
            Ok(::prelude::cel::Value::Null)
          }
        }

        impl ::prelude::TryIntoCel for #target_ident {
          fn try_into_cel(self) -> Result<::prelude::cel::Value, CelError> {
            Ok(::prelude::cel::Value::Null)
          }
        }
      });
    }

    output
  }
}
