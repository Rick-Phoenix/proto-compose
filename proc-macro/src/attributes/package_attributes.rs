use crate::*;

pub fn package_macro_impl(input: TokenStream2) -> syn::Result<TokenStream2> {
  let input_span = input.span();

  let mut const_ident: Option<Ident> = None;
  let mut pkg_name: Option<String> = None;
  let mut include_cel_test = true;

  let parser = syn::meta::parser(|meta| {
    let ident = meta.ident_str()?;

    match ident.as_str() {
      "name" => {
        pkg_name = Some(meta.parse_value::<LitStr>()?.value());
      }
      "no_cel_test" => {
        include_cel_test = false;
      }
      _ => const_ident = Some(meta.ident()?.clone()),
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

  let pkg_name = pkg_name.ok_or_else(|| error_with_span!(input_span, "package name is missing"))?;
  let converted_name = to_snake_case(&pkg_name.replace(".", "_"));

  let test_impl = include_cel_test.then(|| {
    let test_fn_ident = format_ident!("unique_cel_rules_{converted_name}");

    quote! {
      #[cfg(test)]
      #[test]
      fn #test_fn_ident() {
        let pkg = #const_ident.get_package();

        if let Err(e) = pkg.check_unique_cel_rules() {
          panic!("{e}");
        }
      }
    }
  });

  Ok(quote! {
    pub const #const_ident: ::prelude::PackageReference = ::prelude::PackageReference::new(#pkg_name);

    #test_impl
  })
}
