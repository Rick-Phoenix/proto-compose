use crate::*;

pub fn process_file_macro(input: TokenStream2) -> syn::Result<TokenStream2> {
  let input_span = input.span();

  let mut const_ident: Option<Ident> = None;
  let mut file: Option<String> = None;
  let mut package: Option<Path> = None;
  let mut options = TokensOr::<TokenStream2>::new(|| quote! { vec![] });
  let mut extern_path = TokensOr::<LitStr>::new(|| quote! { std::module_path!() });
  let mut imports: Vec<String> = Vec::new();
  let mut extensions = IterTokensOr::<Path>::vec().with_formatter(|items, tokens| {
    tokens
      .extend(quote! { vec![ #(<#items as ::prelude::ProtoExtension>::as_proto_extension()),* ] })
  });

  let parser = syn::meta::parser(|meta| {
    let ident_str = meta.ident_str()?;

    match ident_str.as_str() {
      "file" => {
        file = Some(meta.parse_value::<LitStr>()?.value());
      }
      "package" => {
        package = Some(meta.parse_value::<Path>()?);
      }
      "options" => {
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
      _ => {
        const_ident = Some(meta.ident()?.clone());
      }
    };

    Ok(())
  });

  parser.parse2(input)?;

  let const_ident = const_ident.ok_or_else(|| {
    error_with_span!(
      input_span,
      "Missing const ident (must be the first argument)"
    )
  })?;
  let file = file.ok_or_else(|| error_with_span!(input_span, "Missing `file` attribute"))?;
  let package =
    package.ok_or_else(|| error_with_span!(input_span, "Missing `package` attribute"))?;

  Ok(quote! {
    #[doc(hidden)]
    #[allow(unused)]
    const #const_ident: ::prelude::RegistryPath = ::prelude::RegistryPath {
      file: #file,
      package: #package.name,
      extern_path: #extern_path,
    };

    #[doc(hidden)]
    #[allow(unused)]
    const __PROTO_FILE: ::prelude::RegistryPath = #const_ident;

    ::prelude::inventory::submit! {
      ::prelude::RegistryFile {
        file: __PROTO_FILE.file,
        package: __PROTO_FILE.package,
        options: || #options,
        imports: || vec![ #(#imports),* ],
        extensions: || #extensions
      }
    }
  })
}
