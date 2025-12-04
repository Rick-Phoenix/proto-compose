use crate::*;

pub fn process_service_derive(item: &mut ItemEnum) -> Result<TokenStream2, Error> {
  let ItemEnum {
    attrs,
    ident,
    variants,
    ..
  } = item;

  let mut handlers_tokens: Vec<TokenStream2> = Vec::new();

  let ServiceOrHandlerAttrs {
    name: service_name,
    options: service_options,
    package,
  } = process_service_or_handler_attrs(ident, attrs)?;

  for variant in variants {
    let ServiceOrHandlerAttrs {
      name: handler_name,
      options: handler_options,
      ..
    } = process_service_or_handler_attrs(&variant.ident, &variant.attrs)?;

    let mut request: Option<&Path> = None;
    let mut response: Option<&Path> = None;

    let fields = if let Fields::Named(named) = &variant.fields {
      &named.named
    } else {
      bail!(
        variant,
        "Service variants must have 2 named fields, `request` and `response`"
      );
    };

    for field in fields {
      let field_ident = field
        .ident
        .as_ref()
        .ok_or(spanned_error!(field, "Expected a named field"))?
        .to_string();

      let field_type = match &field.ty {
        Type::Path(type_path) => &type_path.path,
        _ => bail!(&field.ty, "Expected a type path"),
      };

      match field_ident.as_str() {
        "request" => request = Some(field_type),
        "response" => response = Some(field_type),
        _ => bail!(
          variant,
          "Service variants must have 2 named fields, `request` and `response`"
        ),
      };
    }

    let request = request.ok_or(spanned_error!(&variant, "Missing request type"))?;
    let response = response.ok_or(spanned_error!(&variant, "Missing response type"))?;

    let handler_options = tokens_or_default!(handler_options, quote! { vec![] });

    handlers_tokens.push(quote! {
      ::prelude::ServiceHandler {
        name: #handler_name,
        request: #request::path(),
        response: #response::path(),
        options: #handler_options
      }
    });
  }

  let package = package.ok_or(spanned_error!(&ident, "Missing package attribute"))?;
  let service_options = tokens_or_default!(service_options, quote! { vec![] });

  Ok(quote! {
    impl #ident {
      pub fn as_service() -> ::prelude::Service {
        ::prelude::Service {
          name: #service_name,
          package: #package,
          handlers: vec![ #(#handlers_tokens),* ],
          options: #service_options
        }
      }
    }
  })
}
