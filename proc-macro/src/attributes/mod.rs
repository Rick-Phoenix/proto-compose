mod enum_attributes;
mod enum_parsing;
mod enum_variant_attributes;
mod field_attributes;
mod message_attributes;
mod message_parsing;
mod module_attributes;
mod module_enum_attributes;
mod module_enum_variant_attributes;
mod module_field_attributes;
mod module_message_attributes;
pub(crate) mod nested_items;
mod oneof_attributes;
mod oneof_parsing;
mod parsers;
mod reserved_names;
mod reserved_numbers;
mod tag_allocator;

pub use enum_attributes::*;
pub use enum_parsing::*;
pub use enum_variant_attributes::*;
pub use field_attributes::*;
pub use message_attributes::*;
pub use message_parsing::*;
pub use module_attributes::*;
pub use module_enum_attributes::*;
pub use module_enum_variant_attributes::*;
pub use module_field_attributes::*;
pub use module_message_attributes::*;
pub use oneof_attributes::*;
pub use oneof_parsing::*;
pub use parsers::*;
pub use reserved_names::*;
pub use reserved_numbers::*;
pub use tag_allocator::*;

use crate::*;

pub(crate) struct OptionTokens<'a, T: ToTokens> {
  pub item: Option<&'a T>,
}

impl<'a, T: ToTokens> ToTokens for OptionTokens<'a, T> {
  fn to_tokens(&self, tokens: &mut TokenStream2) {
    tokens.extend(if let Some(item) = &self.item {
      quote! { Some(#item) }
    } else {
      quote! { None }
    });
  }
}

impl<'a, T: ToTokens> OptionTokens<'a, T> {
  pub fn new(item: Option<&'a T>) -> OptionTokens<'a, T> {
    Self { item }
  }

  pub fn map_none<F>(&self, func: F) -> TokenStream2
  where
    F: FnOnce(&'a T) -> TokenStream2,
  {
    if let Some(item) = self.item {
      let closure_result = func(item);
      quote! { Some(#closure_result) }
    } else {
      quote! { None }
    }
  }

  pub fn map_empty<F>(&self, func: F) -> TokenStream2
  where
    F: FnOnce(&'a T) -> TokenStream2,
  {
    if let Some(item) = self.item {
      let closure_result = func(item);
      quote! { Some(#closure_result) }
    } else {
      TokenStream2::new()
    }
  }
}
