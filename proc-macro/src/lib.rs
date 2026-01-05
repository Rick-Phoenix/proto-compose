#![allow(
  clippy::single_match,
  clippy::collapsible_if,
  clippy::collapsible_else_if
)]

use std::borrow::Borrow;
use std::{borrow::Cow, fmt::Display, ops::Range};

use attributes::*;
use convert_case::ccase;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, format_ident, quote, quote_spanned};
use syn::{
  Attribute, Error, Expr, Field, Fields, Ident, ItemEnum, ItemStruct, Lit, LitStr, MetaList, Path,
  RangeLimits, Token, Type, Variant, Visibility,
  meta::ParseNestedMeta,
  parse::{Parse, Parser},
  parse_macro_input, parse_quote,
  spanned::Spanned,
  token,
};
use syn_utils::*;

use crate::reflection::reflection_derive;
use crate::{
  common_impls::*, enum_derive::*, extension_derive::*, impls::*, item_cloners::*,
  message_derive::*, message_schema_impl::*, oneof_derive::*, path_utils::*, proto_field::*,
  proto_map::*, proto_types::*, service_derive::*,
};

mod attributes;
mod common_impls;
mod enum_derive;
mod extension_derive;
mod impls;
mod item_cloners;
mod message_derive;
mod message_schema_impl;
mod oneof_derive;
mod oneof_schema_impl;
mod path_utils;
mod proto_field;
mod proto_map;
mod proto_types;
#[cfg(feature = "reflection")]
mod reflection;
mod service_derive;

#[proc_macro_derive(ValidatedMessage, attributes(proto))]
pub fn validated_message_derive(input: TokenStream) -> TokenStream {
  let mut item = parse_macro_input!(input as ItemStruct);

  let validator_impl = match reflection::reflection_derive(&mut item) {
    Ok(imp) => imp,
    Err(e) => {
      let err = e.into_compile_error();
      let fallback_impl = fallback_message_validator_impl(&item.ident);

      quote! {
        #fallback_impl
        #err
      }
    }
  };

  validator_impl.into()
}

#[proc_macro]
pub fn define_proto_file(input: TokenStream) -> TokenStream {
  match process_file_macro(input.into()) {
    Ok(output) => output.into(),
    Err(e) => e.into_compile_error().into(),
  }
}

#[proc_macro]
pub fn proto_package(input: TokenStream) -> TokenStream {
  match package_macro_impl(input.into()) {
    Ok(output) => output.into(),
    Err(e) => e.into_compile_error().into(),
  }
}

#[proc_macro_attribute]
pub fn proto_message(args: TokenStream, input: TokenStream) -> TokenStream {
  let item = parse_macro_input!(input as ItemStruct);

  process_message_derive(item, args.into()).into()
}

#[proc_macro_derive(Message, attributes(proto))]
pub fn message_derive(_input: TokenStream) -> TokenStream {
  TokenStream::new()
}

#[proc_macro_attribute]
pub fn proto_extension(args: TokenStream, input: TokenStream) -> TokenStream {
  let mut item = parse_macro_input!(input as ItemStruct);

  let extra_tokens = match process_extension_derive(args.into(), &mut item) {
    Ok(output) => output,
    Err(e) => return e.to_compile_error().into(),
  };

  quote! {
    #[derive(::prelude::macros::Extension)]
    #item

    #extra_tokens
  }
  .into()
}

#[proc_macro_derive(Extension, attributes(proto))]
pub fn extension_derive(_input: TokenStream) -> TokenStream {
  TokenStream::new()
}

#[proc_macro_attribute]
pub fn proto_service(_args: TokenStream, input: TokenStream) -> TokenStream {
  let item = parse_macro_input!(input as ItemEnum);

  let output = match process_service_derive(&item) {
    Ok(output) => output,
    Err(e) => return e.to_compile_error().into(),
  };

  output.into()
}

#[proc_macro_derive(Service, attributes(proto))]
pub fn service_derive(_input: TokenStream) -> TokenStream {
  TokenStream::new()
}

#[proc_macro_attribute]
pub fn proto_enum(_args: TokenStream, input: TokenStream) -> TokenStream {
  let item = parse_macro_input!(input as ItemEnum);

  process_enum_derive(item).into()
}

#[proc_macro_derive(Enum, attributes(proto))]
pub fn enum_derive(_input: TokenStream) -> TokenStream {
  TokenStream::new()
}

#[proc_macro_attribute]
pub fn proto_oneof(args: TokenStream, input: TokenStream) -> TokenStream {
  let item = parse_macro_input!(input as ItemEnum);

  process_oneof_derive(item, args.into()).into()
}

#[proc_macro_derive(Oneof, attributes(proto))]
pub fn oneof_derive(_input: TokenStream) -> TokenStream {
  TokenStream::new()
}
