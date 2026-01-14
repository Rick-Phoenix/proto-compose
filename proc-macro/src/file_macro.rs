use crate::*;

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

  let inventory_cfg_guard = guard_inventory_on_no_std();

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

    #inventory_cfg_guard
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
  })
}
