use crate::*;

pub fn builder_macro(input: TokenStream2) -> syn::Result<TokenStream2> {
  let parser = PunctuatedItems::<Ident>::parse;
  let idents = parser.parse2(input)?.list;

  let setter_types = idents.iter().map(|id| {
    let ident = format_ident!("Set{id}");
    let others = idents.iter().filter(|id2| id != *id2);

    quote! {
      pub struct #ident<S: State = Empty>(PhantomData<fn() -> S>);


      #[doc(hidden)]
      impl<S: State> State for #ident<S> {
        type #id = Set<members::#id>;
        #(
          type #others = S::#others;
        )*
        const SEALED: sealed::Sealed = sealed::Sealed;
      }
    }
  });

  Ok(quote! {
    use core::marker::PhantomData;

    mod sealed {
      pub(super) struct Sealed;
    }

    pub trait State: Sized {
      #(
        type #idents;
      )*
      #[doc(hidden)]
      const SEALED: sealed::Sealed;
    }

    #[doc(hidden)]
    mod members {
      #(
        pub struct #idents;
      )*
    }

    #[doc(hidden)]
    impl State for Empty {
      #(
        type #idents = Unset<members::#idents>;
      )*
      const SEALED: sealed::Sealed = sealed::Sealed;
    }

    #(#setter_types)*
  })
}
