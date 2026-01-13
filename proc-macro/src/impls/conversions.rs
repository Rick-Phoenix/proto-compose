use crate::*;

fn process_custom_expression(expr: &PathOrClosure, base_ident: &TokenStream2) -> TokenStream2 {
  match expr {
    PathOrClosure::Path(path) => quote! { #path(#base_ident) },
    PathOrClosure::Closure(closure) => {
      quote_spanned! {closure.span()=>
        ::prelude::apply(#base_ident, #closure)
      }
    }
  }
}

pub struct ProtoConversions<'a> {
  pub proxy_ident: &'a Ident,
  pub proto_ident: &'a Ident,
  pub kind: ItemKind,
  pub container_attrs: ContainerAttrs<'a>,
  pub fields: &'a [FieldDataKind],
}

impl ProtoConversions<'_> {
  pub fn generate_proto_conversions(&self) -> TokenStream2 {
    let Self {
      proxy_ident,
      proto_ident,
      kind,
      ..
    } = self;

    let from_proto = self.create_from_proto_impl();
    let into_proto = self.create_into_proto_impl();

    let proxy_trait_impl = if kind.is_message() {
      quote! {
        impl ::prelude::MessageProxy for #proxy_ident {
          type Message = #proto_ident;
        }

        impl ::prelude::ProxiedMessage for #proto_ident {
          type Proxy = #proxy_ident;
        }
      }
    } else {
      quote! {
        impl ::prelude::OneofProxy for #proxy_ident {
          type Oneof = #proto_ident;
        }

        impl ::prelude::ProxiedOneof for #proto_ident {
          type Proxy = #proxy_ident;
        }
      }
    };

    quote! {
      #from_proto
      #into_proto
      #proxy_trait_impl
    }
  }

  fn create_from_proto_impl(&self) -> TokenStream2 {
    let Self {
      proxy_ident,
      proto_ident,
      kind,
      container_attrs,
      fields,
    } = self;

    let custom_from_proto = container_attrs.custom_from_proto_expr();

    let conversion_body = if let Some(from_proto) = custom_from_proto {
      process_custom_expression(from_proto, &quote_spanned! {from_proto.span()=> value })
    } else if fields.is_empty() {
      quote! { unimplemented!() }
    } else {
      let tokens = fields.iter()
        // For oneofs, ignored variants do not map to the original enum
        .filter(|d| !(d.is_ignored() && kind.is_oneof()))
        .map(|d| {
      let field_ident = d.ident();
        let span = field_ident.span();

      let conversion_logic = match d {
        FieldDataKind::Ignored { from_proto, .. } => {
          if let Some(expr) = from_proto {
            match expr {
              // Field is ignored, so we don't pass any args here
              PathOrClosure::Path(path) => quote_spanned! {span=> #path() },
              PathOrClosure::Closure(closure) => {
                let error = error!(closure, "Cannot use a closure for ignored fields");

                error.into_compile_error()
              }
            }
          } else {
            quote_spanned! {span=> Default::default() }
          }
        }
        FieldDataKind::Normal(field_data) => {
          let base_ident = match kind {
            ItemKind::Oneof => quote_spanned! {span=> v },
            ItemKind::Message => {
              quote_spanned! {span=> value.#field_ident }
            }
          };

          if let Some(expr) = field_data.from_proto.as_ref() {
            process_custom_expression(expr, &base_ident)
          } else {
            field_data
              .proto_field
              .default_from_proto(&base_ident)
          }
        }
      };

      match kind {
        ItemKind::Oneof => {
          quote_spanned! {span=> #proto_ident::#field_ident(v) => #proxy_ident::#field_ident(#conversion_logic) }
        }
        ItemKind::Message => quote_spanned! {span=> #field_ident: #conversion_logic },
      }
    });

      match kind {
        ItemKind::Oneof => quote! {
          match value {
            #(#tokens),*
          }
        },
        ItemKind::Message => {
          quote! {
            Self {
              #(#tokens),*
            }
          }
        }
      }
    };

    quote! {
      #[allow(clippy::useless_conversion)]
      impl From<#proto_ident> for #proxy_ident {
        fn from(value: #proto_ident) -> Self {
          #conversion_body
        }
      }
    }
  }

  fn create_into_proto_impl(&self) -> TokenStream2 {
    let Self {
      proxy_ident,
      proto_ident,
      kind,
      container_attrs,
      fields,
    } = self;

    let custom_into_proto = container_attrs.custom_into_proto_expr();

    let conversion_body = if let Some(into_proto) = custom_into_proto {
      process_custom_expression(into_proto, &quote_spanned! {into_proto.span()=> value })
    } else if fields.is_empty() {
      quote! { unimplemented!() }
    } else {
      let tokens = fields
        .iter()
        .filter(|d| !(d.is_ignored() && kind.is_message()))
        .map(|d| match d {
          // This is only for ignored oneof variants
          FieldDataKind::Ignored {
            ident, into_proto, ..
          } => {
            if let Some(expr) = into_proto {
              let conversion = process_custom_expression(expr, &quote_spanned! {ident.span()=> v });

              quote_spanned! {ident.span()=> #proxy_ident::#ident(v) => #conversion }
            } else {
              quote_spanned! {ident.span()=> #proxy_ident::#ident(..) => #proto_ident::default() }
            }
          }
          FieldDataKind::Normal(field_data) => {
            let field_ident = &field_data.ident;
            let span = field_ident.span();

            let base_ident = match kind {
              ItemKind::Oneof => quote_spanned! {span=> v },
              ItemKind::Message => {
                quote_spanned! {span=> value.#field_ident }
              }
            };

            let conversion_logic = if let Some(expr) = field_data.into_proto.as_ref() {
              process_custom_expression(expr, &base_ident)
            } else {
              field_data
                .proto_field
                .default_into_proto(&base_ident)
            };

            match kind {
              ItemKind::Oneof => quote_spanned! {span=>
                #proxy_ident::#field_ident(v) => #proto_ident::#field_ident(#conversion_logic)
              },
              ItemKind::Message => quote_spanned! {span=> #field_ident: #conversion_logic },
            }
          }
        });

      match kind {
        ItemKind::Oneof => quote! {
          match value {
            #(#tokens),*
          }
        },
        ItemKind::Message => {
          quote! {
            Self {
              #(#tokens),*
            }
          }
        }
      }
    };

    quote! {
      #[allow(clippy::useless_conversion)]
      impl From<#proxy_ident> for #proto_ident {
        fn from(value: #proxy_ident) -> Self {
          #conversion_body
        }
      }
    }
  }
}

#[derive(Clone, Copy)]
pub enum ItemKind {
  Oneof,
  Message,
}

impl ItemKind {
  /// Returns `true` if the input item kind is [`Message`].
  ///
  /// [`Message`]: InputItemKind::Message
  #[must_use]
  pub const fn is_message(self) -> bool {
    matches!(self, Self::Message)
  }

  /// Returns `true` if the item kind is [`Oneof`].
  ///
  /// [`Oneof`]: ItemKind::Oneof
  #[must_use]
  pub const fn is_oneof(self) -> bool {
    matches!(self, Self::Oneof)
  }
}
