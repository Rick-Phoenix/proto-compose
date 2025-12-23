use crate::*;

pub struct OneofCheckCtx {
  pub path: TokenStream2,
  pub tags: Vec<i32>,
}

pub fn generate_oneof_tags_check(
  struct_ident: &Ident,
  no_auto_test: bool,
  oneofs: Vec<OneofCheckCtx>,
) -> TokenStream2 {
  if oneofs.is_empty() {
    return TokenStream2::new();
  }

  let ident_str = struct_ident.to_string();

  let mut test_body = TokenStream2::new();

  for oneof in oneofs {
    let OneofCheckCtx { path, tags } = oneof;

    test_body.extend(quote! {
      #path::check_tags(#ident_str, &mut [ #(#tags),* ]);
    });
  }

  let test_impl = quote! {
    #[cfg(test)]
    impl #struct_ident {
      #[track_caller]
      pub fn check_oneofs_tags() {
        #test_body
      }
    }
  };

  let auto_generated_test = (!no_auto_test).then(|| {
    let test_fn_name = format_ident!("{}_oneof_tags_check", ccase!(snake, &ident_str));

    quote! {
      #[cfg(test)]
      #[test]
      fn #test_fn_name() {
        #test_body
      }
    }
  });

  quote! {
    #test_impl
    #auto_generated_test
  }
}
