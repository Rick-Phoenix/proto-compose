use crate::*;

pub struct FallbackImpls<'a> {
  pub orig_ident: &'a Ident,
  pub proto_ident: Option<&'a Ident>,
  pub kind: ItemKind,
}

impl<'a> FallbackImpls<'a> {
  fn proto_ident(&self) -> &'a Ident {
    self.proto_ident.unwrap_or(self.orig_ident)
  }

  pub fn fallback_derive_impls(&self) -> TokenStream2 {
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

    output.extend(match self.kind {
      ItemKind::Oneof => {
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
      ItemKind::Message => {
        quote! {
          impl ::prost::Message for #target_ident {
            fn encoded_len(&self) -> usize {
              unimplemented!()
            }

            fn encode_raw(&self, _: &mut impl ::prost::bytes::buf::BufMut) { unimplemented!() }

            fn merge_field(&mut self, _: u32, _: prost::encoding::WireType, _: &mut impl ::prost::bytes::buf::Buf, _: prost::encoding::DecodeContext) -> std::result::Result<(), prost::DecodeError> { unimplemented!() }

            fn clear(&mut self) {}
          }
        }
      }
    }
    );

    if cfg!(feature = "cel") {
      output.extend(match self.kind {
        ItemKind::Oneof => quote! {
          impl ::prelude::CelOneof for #target_ident {
            #[doc(hidden)]
            fn try_into_cel_recursive(self, depth: usize) -> Result<(String, ::prelude::cel::Value), ::prelude::proto_types::cel::CelConversionError> {
              unimplemented!()
            }
          }

          impl TryFrom<#target_ident> for ::prelude::cel::Value {
            type Error = ::prelude::proto_types::cel::CelConversionError;

            #[inline]
            fn try_from(value: #target_ident) -> Result<Self, Self::Error> {
              unimplemented!()
            }
          }
        },
        ItemKind::Message => quote! {
          impl #target_ident {
            pub fn try_into_cel_recursive(self, _: usize) -> Result<::prelude::cel::Value, ::prelude::proto_types::cel::CelConversionError> {
              unimplemented!()
            }
          }

          impl ::prelude::TryIntoCel for #target_ident {
            fn try_into_cel(self) -> Result<::prelude::cel::Value, ::prelude::CelError> {
              unimplemented!()
            }
          }
        },
      });
    }

    output
  }
}
