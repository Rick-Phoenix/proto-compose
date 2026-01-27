#![allow(
  clippy::single_match,
  clippy::collapsible_if,
  clippy::collapsible_else_if
)]

use std::{borrow::Cow, fmt::Display, ops::Range};

use attributes::*;
use bool_enum::bool_enum;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, format_ident, quote, quote_spanned};
use syn::{
  Attribute, Error, Expr, Field, Fields, Ident, ItemEnum, ItemStruct, Lit, LitBool, LitStr, Meta,
  Path, RangeLimits, Token, Type, Variant, Visibility, bracketed,
  meta::ParseNestedMeta,
  parse::{Parse, Parser},
  parse_macro_input, parse_quote, parse_quote_spanned,
  spanned::Spanned,
  token,
};
use syn_utils::*;

use crate::{
  enum_proc_macro::*, extension_derive::*, file_macro::*, impls::*, item_cloners::*,
  message_proc_macro::*, oneof_proc_macro::*, package_macro::*, path_utils::*, proto_field::*,
  proto_map::*, proto_types::*, service_derive::*,
};

mod attributes;
mod builder_macro;
#[cfg(feature = "cel")]
mod cel_try_into;
#[cfg(feature = "reflection")]
mod enum_derive;
mod enum_proc_macro;
mod extension_derive;
mod field_data;
mod file_macro;
mod impls;
mod item_cloners;
mod message_proc_macro;
mod message_schema_impl;
mod oneof_proc_macro;
mod oneof_schema_impl;
mod package_macro;
mod path_utils;
mod proto_field;
mod proto_map;
mod proto_types;
#[cfg(feature = "reflection")]
mod reflection;
mod service_derive;

#[doc(hidden)]
#[proc_macro_derive(AttrForwarding, attributes(forward))]
pub fn attr_forwarding_derive_test(_: TokenStream) -> TokenStream {
  TokenStream::new()
}

#[cfg(feature = "cel")]
#[proc_macro_derive(CelOneof, attributes(cel))]
pub fn cel_oneof_derive(input: TokenStream) -> TokenStream {
  let item = parse_macro_input!(input as ItemEnum);

  match cel_try_into::derive_cel_value_oneof(&item) {
    Ok(tokens) => tokens.into(),
    Err(e) => e.into_compile_error().into(),
  }
}

#[cfg(feature = "cel")]
#[proc_macro_derive(CelValue, attributes(cel))]
pub fn cel_struct_derive(input: TokenStream) -> TokenStream {
  let item = parse_macro_input!(input as ItemStruct);

  match cel_try_into::derive_cel_value_struct(&item) {
    Ok(tokens) => tokens.into(),
    Err(e) => e.into_compile_error().into(),
  }
}

#[cfg(feature = "reflection")]
#[proc_macro_derive(ValidatedOneof, attributes(proto))]
pub fn validated_oneof_derive(input: TokenStream) -> TokenStream {
  let mut item = parse_macro_input!(input as ItemEnum);

  reflection::reflection_oneof_derive(&mut item).into()
}

#[cfg(feature = "reflection")]
#[proc_macro_derive(ProtoEnum, attributes(proto))]
pub fn enum_derive(input: TokenStream) -> TokenStream {
  let item = parse_macro_input!(input as ItemEnum);

  enum_derive::named_enum_derive(&item).into()
}

#[cfg(feature = "reflection")]
#[proc_macro_derive(ValidatedMessage, attributes(proto))]
pub fn validated_message_derive(input: TokenStream) -> TokenStream {
  let mut item = parse_macro_input!(input as ItemStruct);

  reflection::reflection_message_derive(&mut item).into()
}

#[proc_macro]
pub fn builder_state_macro(input: TokenStream) -> TokenStream {
  match builder_macro::builder_macro(input.into()) {
    Ok(output) => output.into(),
    Err(e) => e.into_compile_error().into(),
  }
}

#[proc_macro]
pub fn define_proto_file(input: TokenStream) -> TokenStream {
  match process_file_macro(input.into()) {
    Ok(output) => output.into(),
    Err(e) => e.into_compile_error().into(),
  }
}

#[proc_macro]
pub fn file_schema(input: TokenStream) -> TokenStream {
  match schema_file_macro(input.into()) {
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

  message_proc_macro(item, args.into()).into()
}

#[doc(hidden)]
#[proc_macro_derive(Message, attributes(proto))]
pub fn message_derive(_input: TokenStream) -> TokenStream {
  TokenStream::new()
}

#[proc_macro_attribute]
pub fn proto_extension(args: TokenStream, input: TokenStream) -> TokenStream {
  let mut item = parse_macro_input!(input as ItemStruct);

  let extra_tokens = match process_extension_derive(args.into(), &mut item) {
    Ok(output) => output,
    Err(e) => e.to_compile_error(),
  };

  quote! {
    #[derive(::prelude::macros::Extension)]
    #item

    #extra_tokens
  }
  .into()
}

#[doc(hidden)]
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

#[doc(hidden)]
#[proc_macro_derive(Service, attributes(proto))]
pub fn service_derive(_input: TokenStream) -> TokenStream {
  TokenStream::new()
}

#[proc_macro_attribute]
pub fn proto_enum(_args: TokenStream, input: TokenStream) -> TokenStream {
  let item = parse_macro_input!(input as ItemEnum);

  enum_proc_macro(item).into()
}

#[doc(hidden)]
#[proc_macro_derive(Enum, attributes(proto))]
pub fn enum_empty_derive(_input: TokenStream) -> TokenStream {
  TokenStream::new()
}

#[proc_macro_attribute]
pub fn proto_oneof(args: TokenStream, input: TokenStream) -> TokenStream {
  let item = parse_macro_input!(input as ItemEnum);

  process_oneof_proc_macro(item, args.into()).into()
}

#[doc(hidden)]
#[proc_macro_derive(Oneof, attributes(proto))]
pub fn oneof_derive(_input: TokenStream) -> TokenStream {
  TokenStream::new()
}
