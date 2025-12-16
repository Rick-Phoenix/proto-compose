#![allow(
  clippy::single_match,
  clippy::collapsible_if,
  clippy::collapsible_else_if
)]

#[macro_use]
mod macros;

use std::{borrow::Cow, collections::HashMap, fmt::Display, ops::Range};

use attributes::*;
use convert_case::ccase;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote, ToTokens};
use syn::{
  parse::{Parse, ParseStream, Parser},
  parse_macro_input, parse_quote,
  punctuated::Punctuated,
  spanned::Spanned,
  token,
  token::{Brace, Struct},
  Attribute, Error, Expr, Field, Fields, Generics, Ident, Item, ItemEnum, ItemFn, ItemMod,
  ItemStruct, Lit, Meta, MetaList, MetaNameValue, Path, RangeLimits, Token, Type, Variant,
  Visibility,
};
use syn_utils::{
  bail, error, error_call_site, error_with_span, filter_attributes, CallOrClosure, ExprExt,
  IdentList, NumList, PathList, PathOrClosure, RustType, StringList, TypeInfo,
};

use crate::{
  conversions::*, enum_derive::*, extension_derive::*, item_cloners::*, message_derive::*,
  message_schema_impl::*, module_processing::*, oneof_derive::*, oneof_schema_impl::*,
  path_utils::*, process_field::*, proto_field::*, proto_map::*, proto_types::*, service_derive::*,
  type_extraction::*,
};

mod conversions;
mod enum_derive;
mod extension_derive;
mod item_cloners;
mod message_derive;
mod message_schema_impl;
mod module_processing;
mod oneof_derive;
mod oneof_schema_impl;
mod path_utils;
mod process_field;
mod proto_field;
mod proto_map;
mod proto_types;
mod service_derive;
mod type_extraction;

mod attributes;

#[proc_macro_attribute]
pub fn proto_message(_args: TokenStream, input: TokenStream) -> TokenStream {
  let mut item = parse_macro_input!(input as ItemStruct);

  if !matches!(item.fields, Fields::Named(_)) {
    return error!(
      &item.ident,
      "The proto_message macro can only be used with structs that have named fields"
    )
    .to_compile_error()
    .into();
  }

  let extra_tokens = match process_message_derive(&mut item) {
    Ok(output) => output,
    Err(e) => e.into_compile_error(),
  };

  quote! {
    #[derive(Message)]
    #item

    #extra_tokens
  }
  .into()
}

#[proc_macro_derive(Message, attributes(proto))]
pub fn message_derive(_input: TokenStream) -> TokenStream {
  TokenStream::new()
}

#[proc_macro_attribute]
pub fn proto_extension(args: TokenStream, input: TokenStream) -> TokenStream {
  let mut item = parse_macro_input!(input as ItemStruct);

  let extra_tokens = match process_extension_derive(args, &mut item) {
    Ok(output) => output,
    Err(e) => return e.to_compile_error().into(),
  };

  quote! {
    #[derive(Extension)]
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

  let output = match process_service_derive(item) {
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
  let mut item = parse_macro_input!(input as ItemEnum);

  let extra_tokens = match process_enum_derive(&mut item) {
    Ok(output) => output,
    Err(e) => return e.to_compile_error().into(),
  };

  quote! {
    #[derive(Enum)]
    #item

    #extra_tokens
  }
  .into()
}

#[proc_macro_derive(Enum, attributes(proto))]
pub fn enum_derive(_input: TokenStream) -> TokenStream {
  TokenStream::new()
}

#[proc_macro_attribute]
pub fn proto_oneof(_args: TokenStream, input: TokenStream) -> TokenStream {
  let mut item = parse_macro_input!(input as ItemEnum);

  let extra_tokens = match process_oneof_derive(&mut item) {
    Ok(output) => output,
    Err(e) => return e.to_compile_error().into(),
  };

  quote! {
    #[derive(Oneof)]
    #item

    #extra_tokens
  }
  .into()
}

#[proc_macro_derive(Oneof, attributes(proto))]
pub fn oneof_derive(_input: TokenStream) -> TokenStream {
  TokenStream::new()
}

#[proc_macro_attribute]
pub fn proto_module(attrs: TokenStream, input: TokenStream) -> TokenStream {
  let module = parse_macro_input!(input as ItemMod);

  let module_attrs = parse_macro_input!(attrs as ModuleAttrs);

  match process_module_items(module_attrs, module) {
    Ok(processed_module) => processed_module.into_token_stream().into(),
    Err(e) => e.to_compile_error().into(),
  }
}
