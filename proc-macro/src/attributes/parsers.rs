use itertools::Either;
use syn::LitInt;

use crate::*;

pub struct NumList {
  pub list: Vec<i32>,
}

impl Parse for NumList {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let items = Punctuated::<LitInt, Token![,]>::parse_terminated(input)?;

    let mut list: Vec<i32> = Vec::new();

    for item in items {
      list.push(item.base10_parse()?);
    }

    Ok(Self { list })
  }
}

pub fn get_proto_args(attr: &Attribute) -> Result<impl Iterator<Item = Meta>, Error> {
  if attr.path().is_ident("proto") {
    Ok(Either::Left(
      attr
        .parse_args::<PunctuatedParser<Meta>>()?
        .inner
        .into_iter(),
    ))
  } else {
    Ok(Either::Right(std::iter::empty::<Meta>()))
  }
}

pub struct PunctuatedParser<T: Parse> {
  pub inner: Punctuated<T, Token![,]>,
}

impl<T: Parse> Parse for PunctuatedParser<T> {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let inner = Punctuated::parse_terminated(input)?;

    Ok(Self { inner })
  }
}

pub fn extract_i32(expr: &Expr) -> Result<i32, Error> {
  if let Expr::Lit(expr_lit) = expr && let Lit::Int(value) = &expr_lit.lit {
    Ok(value.base10_parse()?)
  } else {
    Err(spanned_error!(expr, "Expected an integer literal"))
  }
}

#[derive(Default, Debug)]
pub(crate) struct ProtoOptions(pub Option<TokenStream2>);

impl ToTokens for ProtoOptions {
  fn to_tokens(&self, tokens: &mut TokenStream2) {
    tokens.extend(if let Some(opts) = &self.0 {
      quote! { #opts }
    } else {
      quote! { vec![] }
    });
  }
}

pub struct StringList {
  pub list: Vec<String>,
}

impl Parse for StringList {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let items = Punctuated::<LitStr, Token![,]>::parse_terminated(input)?;

    let list: Vec<String> = items.into_iter().map(|lit_str| lit_str.value()).collect();

    Ok(Self { list })
  }
}

pub fn extract_string_lit(expr: &Expr) -> Result<String, Error> {
  if let Expr::Lit(expr_lit) = expr && let Lit::Str(value) = &expr_lit.lit {
    Ok(value.value())
  } else {
    Err(spanned_error!(expr, "Expected a string literal"))
  }
}

pub fn extract_path(expr: Expr) -> Result<Path, Error> {
  if let Expr::Path(path) = expr {
    Ok(path.path)
  } else {
    Err(spanned_error!(expr, "Expected a path"))
  }
}
