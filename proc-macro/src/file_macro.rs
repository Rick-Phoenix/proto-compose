use crate::*;
use syn::{braced, bracketed, custom_keyword, parse::ParseStream};

enum MessageExpr {
  Single(Path),
  Nested {
    path: Path,
    nested_messages: Vec<Self>,
    nested_enums: Vec<Path>,
  },
}

impl MessageExpr {
  const fn path(&self) -> &Path {
    match self {
      Self::Single(path) | Self::Nested { path, .. } => path,
    }
  }
}

impl ToTokens for MessageExpr {
  fn to_tokens(&self, tokens: &mut TokenStream2) {
    let path = self.path();

    tokens.extend(quote! { <#path as ::prelude::ProtoMessage>::proto_schema() });

    if let Self::Nested {
      nested_messages,
      nested_enums,
      ..
    } = self
    {
      if !nested_messages.is_empty() {
        tokens.extend(quote! {
          .with_nested_messages([ #(#nested_messages),* ])
        });
      }

      if !nested_enums.is_empty() {
        tokens.extend(quote! {
          .with_nested_enums([ #(#nested_enums::proto_schema()),* ])
        });
      }
    }
  }
}

impl Parse for MessageExpr {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let path: Path = input.parse()?;

    if input.peek(Token![=]) {
      input.parse::<Token![=]>()?;

      let content;
      braced!(content in input);

      let mut nested_messages = Vec::new();
      let mut nested_enums = Vec::new();

      while !content.is_empty() {
        let lookahead = content.lookahead1();

        if lookahead.peek(messages) {
          content.parse::<messages>()?;
          content.parse::<Token![=]>()?;

          nested_messages = parse_bracketed::<PunctuatedItems<Self>>(&content)?.list;
        } else if lookahead.peek(enums) {
          content.parse::<enums>()?;
          content.parse::<Token![=]>()?;

          nested_enums = parse_bracketed::<PathList>(&content)?.list;
        } else if lookahead.peek(Token![,]) {
          content.parse::<Token![,]>()?;
        } else {
          return Err(lookahead.error());
        }
      }

      Ok(Self::Nested {
        path,
        nested_messages,
        nested_enums,
      })
    } else {
      Ok(Self::Single(path))
    }
  }
}

custom_keyword!(messages);
custom_keyword!(enums);

fn parse_bracketed<T: Parse>(input: ParseStream) -> syn::Result<T> {
  let content;
  bracketed!(content in input);
  content.parse::<T>()
}

pub fn schema_file_macro(input: TokenStream2) -> syn::Result<TokenStream2> {
  let mut name: Option<String> = None;
  let mut imports: Vec<String> = Vec::new();
  let mut options = TokenStreamOr::new(|_| quote! { ::prelude::vec![] });
  let mut extensions: Vec<Path> = Vec::new();
  let mut edition = TokenStreamOr::new(|_| quote! { ::prelude::Edition::Proto3 });
  let mut messages: Vec<MessageExpr> = Vec::new();
  let mut enums: Vec<Path> = Vec::new();
  let mut services: Vec<Path> = Vec::new();

  let parser = syn::meta::parser(|meta| {
    let ident_str = meta.ident_str()?;

    match ident_str.as_str() {
      "messages" => {
        messages = parse_bracketed::<PunctuatedItems<MessageExpr>>(meta.value()?)?.list;
      }
      "enums" => {
        enums = parse_bracketed::<PathList>(meta.value()?)?.list;
      }
      "services" => {
        services = parse_bracketed::<PathList>(meta.value()?)?.list;
      }
      "name" => {
        name = Some(meta.parse_value::<LitStr>()?.value());
      }
      "options" => {
        options.span = meta.input.span();
        options.set(meta.expr_value()?.into_token_stream());
      }
      "imports" => {
        imports = meta.parse_list::<StringList>()?.list;
      }
      "extensions" => {
        extensions = parse_bracketed::<PathList>(meta.value()?)?.list;
      }
      "edition" => {
        edition.set(meta.parse_value::<Path>()?.into_token_stream());
      }
      _ => return Err(meta.error("Unknown attribute")),
    };

    Ok(())
  });

  parser.parse2(input)?;

  Ok(quote! {
    {
      let mut file = ::prelude::ProtoFile::new(#name, "package");

      file
        .with_messages([ #(#messages),* ])
        .with_enums([ #(#enums::proto_schema()),* ])
        .with_services([ #(#services::as_proto_service()),* ])
        .with_imports([ #(#imports),* ])
        .with_edition(#edition)
        .with_extensions([ #(#extensions::as_proto_extension()),* ])
        .with_options(#options);

      file
    }
  })
}

pub fn process_file_macro(input: TokenStream2) -> syn::Result<TokenStream2> {
  let mut const_ident: Option<Ident> = None;
  let mut name: Option<String> = None;
  let mut package: Option<Path> = None;
  let mut options = TokenStreamOr::new(|_| quote! { ::prelude::vec![] });
  let mut extern_path =
    TokensOr::<LitStr>::new(|span| quote_spanned! (span=> core::module_path!()));
  let mut imports: Vec<String> = Vec::new();
  let mut extensions = IterTokensOr::<Path>::new(
    |_| quote! { ::prelude::vec![] },
    |_, items| {
      quote! { ::prelude::vec![ #(<#items as ::prelude::ProtoExtension>::as_proto_extension()),* ] }
    },
  );
  let mut edition = TokenStreamOr::new(|_| quote! { ::prelude::Edition::Proto3 });

  let parser = syn::meta::parser(|meta| {
    let ident_str = meta.ident_str()?;

    match ident_str.as_str() {
      "name" => {
        name = Some(meta.parse_value::<LitStr>()?.value());
      }
      "package" => {
        package = Some(meta.parse_value::<Path>()?);
      }
      "options" => {
        options.span = meta.input.span();
        options.set(meta.expr_value()?.into_token_stream());
      }
      "extern_path" => {
        extern_path.set(meta.parse_value::<LitStr>()?);
      }
      "imports" => {
        imports = meta.parse_list::<StringList>()?.list;
      }
      "extensions" => {
        extensions.set(meta.parse_list::<PathList>()?.list);
      }
      "edition" => {
        edition.set(meta.parse_value::<Path>()?.into_token_stream());
      }
      _ => {
        const_ident = Some(meta.ident()?.clone());
      }
    };

    Ok(())
  });

  parser.parse2(input)?;

  let const_ident = const_ident
    .ok_or_else(|| error_call_site!("Missing const ident (must be the first argument)"))?;
  let file = name.ok_or_else(|| error_call_site!("Missing `file` attribute"))?;
  let package = package.ok_or_else(|| error_call_site!("Missing `package` attribute"))?;

  let inventory_call = has_inventory_feat().then(|| {
    quote! {
      ::prelude::inventory::submit! {
        ::prelude::RegistryFile {
          name: __PROTO_FILE.name,
          package: __PROTO_FILE.package,
          edition: #edition,
          options: || #options.into_iter().collect(),
          imports: || ::prelude::vec![ #(#imports),* ],
          extensions: || #extensions
        }
      }
    }
  });

  Ok(quote! {
    #[doc(hidden)]
    #[allow(unused)]
    const #const_ident: ::prelude::FileReference = ::prelude::FileReference {
      name: #file,
      package: #package.name,
      extern_path: #extern_path,
    };

    #[doc(hidden)]
    #[allow(unused)]
    const __PROTO_FILE: ::prelude::FileReference = #const_ident;

    #inventory_call
  })
}
