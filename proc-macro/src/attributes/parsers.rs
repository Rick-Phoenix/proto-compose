use syn::LitInt;

use crate::*;

pub fn parse_call(expr: Expr) -> Result<ExprCall, Error> {
  if let Expr::Call(call) = expr {
    Ok(call)
  } else {
    Err(spanned_error!(expr, "Expected a function call"))
  }
}

#[derive(Debug, Clone)]
pub enum PathOrClosure {
  Path(Path),
  Closure(ExprClosure),
}

impl ToTokens for PathOrClosure {
  fn to_tokens(&self, tokens: &mut TokenStream2) {
    let output = match self {
      PathOrClosure::Path(path) => path.to_token_stream(),
      PathOrClosure::Closure(expr_closure) => expr_closure.to_token_stream(),
    };

    tokens.extend(output);
  }
}

pub fn parse_path_or_closure(expr: Expr) -> Result<PathOrClosure, Error> {
  match expr {
    Expr::Closure(closure) => Ok(PathOrClosure::Closure(closure)),
    Expr::Path(expr_path) => Ok(PathOrClosure::Path(expr_path.path)),
    _ => Err(spanned_error!(expr, "Expected a path or a closure")),
  }
}

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
