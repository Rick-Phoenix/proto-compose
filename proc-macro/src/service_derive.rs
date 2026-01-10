use crate::*;

struct HandlerCtx {
  name: String,
  span: Span,
  request: TokenStream2,
  response: TokenStream2,
  options: TokensOr<TokenStream2>,
  deprecated: bool,
}

pub fn process_service_derive(item: &ItemEnum) -> Result<TokenStream2, Error> {
  let ItemEnum {
    attrs,
    ident,
    variants,
    vis,
    ..
  } = item;

  let mut handlers_data: Vec<HandlerCtx> = Vec::new();

  let ServiceOrHandlerAttrs {
    options: service_options,
    deprecated,
  } = process_service_or_handler_attrs(attrs)?;

  let service_name = to_pascal_case(&ident.to_string());

  for variant in variants {
    let ServiceOrHandlerAttrs {
      options: handler_options,
      deprecated,
    } = process_service_or_handler_attrs(&variant.attrs)?;

    let handler_name = variant.ident.to_string();

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
        .ok_or_else(|| error!(field, "Expected a named field"))?
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

    let request = request
      .ok_or_else(|| error!(&variant, "Missing request type"))?
      .to_token_stream();
    let response = response
      .ok_or_else(|| error!(&variant, "Missing response type"))?
      .to_token_stream();

    handlers_data.push(HandlerCtx {
      name: handler_name,
      request,
      response,
      options: handler_options,
      deprecated,
      span: variant.ident.span(),
    });
  }

  let handlers_tokens = handlers_data.iter().map(|data| {
    let HandlerCtx {
      name,
      request,
      response,
      options,
      deprecated,
      span,
    } = data;

    let options_tokens = options_tokens(*span, options, *deprecated);

    quote_spanned! {*span=>
      ::prelude::ServiceHandler {
        name: #name,
        request: <#request as ::prelude::ProtoMessage>::proto_path(),
        response: <#response as ::prelude::ProtoMessage>::proto_path(),
        options: #options_tokens.into_iter().collect()
      }
    }
  });

  let options_tokens = options_tokens(Span::call_site(), &service_options, deprecated);

  Ok(quote! {
    #[derive(::prelude::macros::Service)]
    #vis struct #ident;

    ::prelude::inventory::submit! {
      ::prelude::RegistryService {
        package: __PROTO_FILE.package,
        service: || #ident::as_proto_service()
      }
    }

    impl ::prelude::ProtoService for #ident {
      fn as_proto_service() -> ::prelude::Service {
        ::prelude::Service {
          name: #service_name,
          file: __PROTO_FILE.name,
          package: __PROTO_FILE.package,
          handlers: vec![ #(#handlers_tokens),* ],
          options: #options_tokens.into_iter().collect()
        }
      }
    }
  })
}
