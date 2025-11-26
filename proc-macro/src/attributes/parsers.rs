use std::{fmt::Display, str::FromStr};

use itertools::Either;
use syn::LitInt;

use crate::*;

#[derive(Debug, Clone)]
pub enum ProtoMapKeys {
  String,
  Int32,
}

impl From<ProtoMapKeys> for ProtoType {
  fn from(value: ProtoMapKeys) -> Self {
    match value {
      ProtoMapKeys::String => Self::String,
      ProtoMapKeys::Int32 => Self::Int32,
    }
  }
}

impl FromStr for ProtoMapKeys {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let output = match s {
      "String" => Self::String,
      "i32" => Self::Int32,
      _ => return Err(format!("Unrecognized map key {s}")),
    };

    Ok(output)
  }
}

#[derive(Debug, Clone)]
pub enum ProtoMapValues {
  String,
  Int32,
  Enum(Path),
  Message,
}

impl Display for ProtoMapKeys {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ProtoMapKeys::String => write!(f, ""),
      ProtoMapKeys::Int32 => write!(f, ""),
    }
  }
}

impl FromStr for ProtoMapValues {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let output = match s {
      "String" => Self::String,
      "i32" => Self::Int32,
      _ => return Err(format!("Unrecognized map value type {s}")),
    };

    Ok(output)
  }
}

impl Display for ProtoMapValues {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ProtoMapValues::String => write!(f, "string"),
      ProtoMapValues::Int32 => write!(f, "int32"),
      ProtoMapValues::Enum(path) => write!(f, "enumeration({})", path.to_token_stream()),
      ProtoMapValues::Message => write!(f, "message"),
    }
  }
}

#[derive(Debug, Clone)]
pub struct ProtoMap {
  pub keys: ProtoMapKeys,
  pub values: ProtoMapValues,
}

impl ToTokens for ProtoMap {
  fn to_tokens(&self, tokens: &mut TokenStream2) {
    let map_str = format!("{}, {}", self.keys, self.values);

    let output = quote! { map = #map_str };

    tokens.extend(output);
  }
}

// impl Parse for ProtoMap {
//   fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
//     let idents = Punctuated::<Ident, Token![,]>::parse_terminated(input)?;
//
//     let keys_ident = idents.first().ok_or(input.error("Expected two idents"))?.to_string();
//     let values_ident = idents.last().ok_or(input.error("Expected two idents"))?.to_string();
//
//     let keys = ProtoMapKeys::from_str(&keys_ident).map_err(|e| input.error(e))?;
//
//     match values_ident.as_str() {
//       ""
//     }
//     let values = ProtoMapValues::from_str(&values_ident.to_string()).map_err(|e| input.error(e))?;
//
//     Ok(Self {
//       keys,
//       values,
//     })
//   }
// }

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
