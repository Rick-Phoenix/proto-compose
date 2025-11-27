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

impl ProtoMapKeys {
  pub fn validator_target_type(&self) -> TokenStream2 {
    match self {
      ProtoMapKeys::String => quote! { String },
      ProtoMapKeys::Int32 => quote! { i32 },
    }
  }

  pub fn output_proto_type(&self) -> TokenStream2 {
    match self {
      ProtoMapKeys::String => quote! { String },
      ProtoMapKeys::Int32 => quote! { i32 },
    }
  }

  pub fn as_proto_type_trait_target(&self) -> TokenStream2 {
    self.output_proto_type()
  }
}

impl FromStr for ProtoMapKeys {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let output = match s {
      "String" | "string" => Self::String,
      "i32" => Self::Int32,
      _ => return Err(format!("Unrecognized map key {s}")),
    };

    Ok(output)
  }
}

impl ProtoMapKeys {
  pub fn from_path(path: &Path) -> Result<Self, Error> {
    let ident = path.get_ident().ok_or(spanned_error!(
      path,
      format!(
        "Type {} is not a supported map key primitive",
        path.to_token_stream()
      )
    ))?;
    let ident_as_str = ident.to_string();

    Self::from_str(&ident_as_str).map_err(|_| {
      spanned_error!(
        path,
        format!("Type {} is not a supported map key primitive", ident_as_str)
      )
    })
  }
}

#[derive(Debug, Clone)]
pub enum ProtoMapValues {
  String,
  Int32,
  Enum(Option<Path>),
  Message(MessagePath),
}

impl ProtoMapValues {
  pub fn validator_target_type(&self) -> TokenStream2 {
    match self {
      ProtoMapValues::String => quote! { String },
      ProtoMapValues::Int32 => quote! { i32 },
      ProtoMapValues::Enum(_) => quote! { GenericProtoEnum },
      ProtoMapValues::Message(_) => quote! { GenericMessage },
    }
  }

  pub fn output_proto_type(&self) -> TokenStream2 {
    match self {
      ProtoMapValues::String => quote! { String },
      ProtoMapValues::Int32 => quote! { i32 },
      ProtoMapValues::Enum(_) => quote! { i32 },
      ProtoMapValues::Message(path) => path.to_token_stream(),
    }
  }

  pub fn as_proto_type_trait_target(&self) -> TokenStream2 {
    match self {
      ProtoMapValues::String => quote! { String },
      ProtoMapValues::Int32 => quote! { i32 },
      ProtoMapValues::Enum(path) => quote! { #path },
      ProtoMapValues::Message(path) => quote! { #path },
    }
  }
}

impl ProtoMapValues {
  pub fn from_path(path: &Path) -> Result<Self, Error> {
    let ident = path.get_ident().ok_or(spanned_error!(
      path,
      format!(
        "Type {} is not a supported map value primitive",
        path.to_token_stream()
      )
    ))?;
    let ident_as_str = ident.to_string();

    Self::from_str(&ident_as_str).map_err(|_| {
      spanned_error!(
        path,
        format!(
          "Type {} is not a supported map value primitive",
          ident_as_str
        )
      )
    })
  }
}

impl Display for ProtoMapKeys {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ProtoMapKeys::String => write!(f, "string"),
      ProtoMapKeys::Int32 => write!(f, "int32"),
    }
  }
}

impl FromStr for ProtoMapValues {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let output = match s {
      "String" => Self::String,
      "i32" | "int32" => Self::Int32,
      "message" => Self::Message(MessagePath::None),
      "enum_" => Self::Enum(None),
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
      ProtoMapValues::Message(_) => write!(f, "message"),
    }
  }
}

#[derive(Debug, Clone)]
pub struct ProtoMap {
  pub keys: ProtoMapKeys,
  pub values: ProtoMapValues,
}

impl ProtoMap {
  pub fn validator_target_type(&self) -> TokenStream2 {
    let keys = self.keys.validator_target_type();
    let values = self.values.validator_target_type();

    quote! { ProtoMap<#keys, #values> }
  }

  pub fn output_proto_type(&self) -> TokenStream2 {
    let keys = self.keys.output_proto_type();
    let values = self.values.output_proto_type();

    quote! { HashMap<#keys, #values> }
  }

  pub fn as_prost_attr_type(&self) -> TokenStream2 {
    let map_str = format!("{}, {}", self.keys, self.values);

    quote! { map = #map_str }
  }

  pub fn as_proto_type_trait_target(&self) -> TokenStream2 {
    let keys = self.keys.as_proto_type_trait_target();
    let values = self.values.as_proto_type_trait_target();

    quote! { HashMap<#keys, #values> }
  }
}

impl Parse for ProtoMap {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let metas = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;

    if metas.len() != 2 {
      return Err(input.error("Expected a list of two items"));
    }

    let keys_path = metas.first().unwrap().require_path_only()?;
    let keys = ProtoMapKeys::from_path(keys_path)?;

    let values = match metas.last().unwrap() {
      Meta::Path(path) => ProtoMapValues::from_path(path)?,
      Meta::List(list) => {
        let list_ident = ident_string!(list.path);

        match list_ident.as_str() {
          "message" => {
            let message_path = list.parse_args::<Path>()?;

            let path_type = if message_path.is_ident("suffixed") {
              MessagePath::Suffixed
            } else {
              MessagePath::Path(message_path)
            };

            ProtoMapValues::Message(path_type)
          }
          "enum_" => {
            let path = list.parse_args::<Path>()?;
            ProtoMapValues::Enum(Some(path))
          }
          _ => return Err(input.error("Unrecognized value list")),
        }
      }
      _ => return Err(input.error("Expected the values to be a list or path")),
    };

    Ok(Self { keys, values })
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
