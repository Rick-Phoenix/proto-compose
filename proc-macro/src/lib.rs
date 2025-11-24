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
  token::{Brace, Paren, Semi, Struct},
  Attribute, Data, DeriveInput, Error, Expr, ExprClosure, Field, Fields, FieldsUnnamed, Generics,
  Ident, Item, ItemEnum, ItemFn, ItemMod, ItemStruct, Lit, LitStr, Meta, Path, RangeLimits, Token,
  Type, Variant, Visibility,
};
use type_extraction::*;

use crate::{
  enum_derive::process_enum_derive, message_derive::process_message_derive,
  oneof_derive::process_oneof_derive,
};

mod enum_derive;
mod message_derive;
mod oneof_derive;
mod type_extraction;

mod attributes;

#[proc_macro_derive(Message, attributes(proto))]
pub fn message_derive(input: TokenStream) -> TokenStream {
  let tokens = parse_macro_input!(input as DeriveInput);

  match process_message_derive(tokens) {
    Ok(output) => output.into(),
    Err(e) => e.to_compile_error().into(),
  }
}

#[proc_macro_derive(Enum, attributes(proto))]
pub fn enum_derive(input: TokenStream) -> TokenStream {
  let tokens = parse_macro_input!(input as DeriveInput);

  match process_enum_derive(tokens) {
    Ok(output) => output.into(),
    Err(e) => e.to_compile_error().into(),
  }
}

#[proc_macro_derive(Oneof, attributes(proto))]
pub fn oneof_derive(input: TokenStream) -> TokenStream {
  let tokens = parse_macro_input!(input as DeriveInput);

  match process_oneof_derive(tokens) {
    Ok(output) => output.into(),
    Err(e) => e.to_compile_error().into(),
  }
}

#[proc_macro_attribute]
pub fn proto_module(attrs: TokenStream, input: TokenStream) -> TokenStream {
  let mut module = parse_macro_input!(input as ItemMod);

  let ModuleAttrs { file, package } = parse_macro_input!(attrs as ModuleAttrs);

  let file_attribute: Attribute = parse_quote! { #[proto(file = #file, package = #package)] };

  if let Some((_, content)) = &mut module.content {
    let TopLevelItemsTokens {
      top_level_messages,
      top_level_enums,
    } = process_module_items2(file_attribute, content).unwrap();

    let aggregator_fn: ItemFn = parse_quote! {
      pub fn proto_file() -> ProtoFile {
        let mut file = ProtoFile {
          name: #file.into(),
          package: #package.into(),
          ..Default::default()
        };

        file.add_messages([ #top_level_messages ]);
        file.add_enums([ #top_level_enums ]);

        file
      }
    };

    content.push(Item::Fn(aggregator_fn));
  }

  quote!(#module).into()
}
