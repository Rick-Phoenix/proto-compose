#[macro_use]
mod macros;

use std::{collections::HashMap, ops::Range, rc::Rc};

use attributes::*;
pub(crate) use convert_case::ccase;
use proc_macro::TokenStream;
use proc_macro2::Span;
pub(crate) use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
  parse::Parse,
  parse_macro_input, parse_quote,
  punctuated::Punctuated,
  token,
  token::{Brace, Struct},
  Attribute, Data, DeriveInput, Error, Expr, ExprClosure, Field, Fields, Generics, Ident, Item,
  ItemEnum, ItemFn, ItemMod, ItemStruct, Lit, LitStr, Meta, Path, RangeLimits, Token, Type,
  Variant, Visibility,
};
use type_extraction::*;

use crate::{
  enum_derive::*, message_derive::*, module_processing::*, oneof_derive::*, type_extraction::*,
};

mod enum_derive;
mod message_derive;
mod module_processing;
mod oneof_derive;
mod type_extraction;

mod attributes;

#[proc_macro_attribute]
pub fn proto_message(_args: TokenStream, input: TokenStream) -> TokenStream {
  let mut item = parse_macro_input!(input as ItemStruct);

  let extra_tokens = match process_message_derive(&mut item) {
    Ok(output) => output,
    Err(e) => return e.to_compile_error().into(),
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
    Ok(processed_module) => quote! { #processed_module }.into(),
    Err(e) => e.to_compile_error().into(),
  }
}
