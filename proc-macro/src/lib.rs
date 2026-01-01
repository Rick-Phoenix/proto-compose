#![allow(
  clippy::single_match,
  clippy::collapsible_if,
  clippy::collapsible_else_if
)]

#[macro_use]
mod macros;

use std::borrow::Borrow;
use std::{borrow::Cow, fmt::Display, ops::Range};

use attributes::*;
use convert_case::ccase;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, format_ident, quote};
use syn::{
  Attribute, Error, Expr, Field, Fields, Ident, ItemEnum, ItemStruct, Lit, LitStr, Meta, MetaList,
  Path, RangeLimits, Token, Type, Variant, Visibility,
  meta::ParseNestedMeta,
  parse::{Parse, Parser},
  parse_macro_input, parse_quote,
  punctuated::Punctuated,
  spanned::Spanned,
  token,
};
use syn_utils::*;

use crate::{
  common_impls::*, enum_derive::*, extension_derive::*, impls::*, item_cloners::*,
  message_derive::*, message_schema_impl::*, oneof_derive::*, oneof_schema_impl::*, path_utils::*,
  proto_field::*, proto_map::*, proto_types::*, service_derive::*,
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
mod service_derive;

#[proc_macro]
pub fn proto_package(input: TokenStream) -> TokenStream {
  match package_macro_impl(input.into()) {
    Ok(output) => output.into(),
    Err(e) => e.into_compile_error().into(),
  }
}

#[proc_macro_attribute]
pub fn proto_message(args: TokenStream, input: TokenStream) -> TokenStream {
  let mut item = parse_macro_input!(input as ItemStruct);

  let mut macro_attrs = MessageMacroAttrs::default();

  let parser = syn::meta::parser(|meta| {
    if let Some(ident) = meta.path.get_ident() {
      let ident = ident.to_string();

      match ident.as_str() {
        "direct" => macro_attrs.is_direct = true,
        "no_auto_test" => macro_attrs.no_auto_test = true,
        "extern_path" => macro_attrs.extern_path = Some(meta.parse_value::<LitStr>()?.value()),
        _ => {}
      };
    }

    Ok(())
  });

  parse_macro_input!(args with parser);

  if !matches!(item.fields, Fields::Named(_)) {
    return error!(
      &item.ident,
      "The proto_message macro can only be used with structs that have named fields"
    )
    .to_compile_error()
    .into();
  }

  let extra_tokens = match process_message_derive(&mut item, macro_attrs) {
    Ok(output) => output,
    Err(e) => e.into_compile_error(),
  };

  quote! {
    #[derive(::proc_macro_impls::Message)]
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

  let extra_tokens = match process_extension_derive(args.into(), &mut item) {
    Ok(output) => output,
    Err(e) => return e.to_compile_error().into(),
  };

  quote! {
    #[derive(::proc_macro_impls::Extension)]
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
    #[repr(i32)]
    #[derive(::prelude::prost::Enumeration, ::proc_macro_impls::Enum, Hash, PartialEq, Eq, Debug, Clone, Copy)]
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
pub fn proto_oneof(args: TokenStream, input: TokenStream) -> TokenStream {
  let mut item = parse_macro_input!(input as ItemEnum);

  let mut is_direct = false;

  let parser = syn::meta::parser(|meta| {
    if meta.path.is_ident("direct") {
      is_direct = true;
    }

    Ok(())
  });

  parse_macro_input!(args with parser);

  let extra_tokens = match process_oneof_derive(&mut item, is_direct) {
    Ok(output) => output,
    Err(e) => return e.to_compile_error().into(),
  };

  quote! {
    #[derive(::proc_macro_impls::Oneof)]
    #item

    #extra_tokens
  }
  .into()
}

#[proc_macro_derive(Oneof, attributes(proto))]
pub fn oneof_derive(_input: TokenStream) -> TokenStream {
  TokenStream::new()
}
