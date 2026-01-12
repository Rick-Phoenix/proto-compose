use syn::LitBool;
use syn_utils::PunctuatedItems;

use crate::*;

#[derive(Default)]
pub struct MessageAttrs {
  pub reserved_names: Vec<String>,
  pub reserved_numbers: ReservedNumbers,
  pub options: TokensOr<TokenStream2>,
  pub name: ParsedStr,
  pub parent_message: Option<Ident>,
  pub from_proto: Option<PathOrClosure>,
  pub into_proto: Option<PathOrClosure>,
  pub shadow_derives: Option<MetaList>,
  pub cel_rules: IterTokensOr<TokenStream2>,
  pub is_proxied: bool,
  pub no_auto_test: bool,
  pub extern_path: Option<ParsedStr>,
  pub deprecated: bool,
}

impl MessageAttrs {
  pub fn has_custom_conversions(&self) -> bool {
    self.from_proto.is_some() && self.into_proto.is_some()
  }
}

#[derive(Default)]
pub struct MessageMacroArgs {
  pub is_proxied: bool,
  pub no_auto_test: bool,
  pub extern_path: Option<ParsedStr>,
}

impl MessageMacroArgs {
  pub fn parse(macro_args: TokenStream2) -> syn::Result<Self> {
    let mut is_proxied = false;
    let mut no_auto_test = false;
    let mut extern_path: Option<ParsedStr> = None;

    let parser = syn::meta::parser(|meta| {
      if let Some(ident) = meta.path.get_ident() {
        let ident = ident.to_string();

        match ident.as_str() {
          "proxied" => is_proxied = true,
          "no_auto_test" => no_auto_test = true,
          "extern_path" => extern_path = Some(meta.parse_value::<ParsedStr>()?),
          _ => {}
        };
      }

      Ok(())
    });

    parser.parse2(macro_args)?;

    Ok(Self {
      is_proxied,
      no_auto_test,
      extern_path,
    })
  }
}

pub fn process_message_attrs(
  struct_ident: &Ident,
  macro_args: MessageMacroArgs,
  attrs: &[Attribute],
) -> Result<MessageAttrs, Error> {
  let mut reserved_names: Vec<String> = Vec::new();
  let mut reserved_numbers = ReservedNumbers::default();
  let mut options = TokensOr::<TokenStream2>::vec();
  let mut proto_name: Option<ParsedStr> = None;
  let mut from_proto: Option<PathOrClosure> = None;
  let mut into_proto: Option<PathOrClosure> = None;
  let mut shadow_derives: Option<MetaList> = None;
  let mut cel_rules = IterTokensOr::<TokenStream2>::vec();
  let mut parent_message: Option<Ident> = None;
  let mut deprecated = false;

  for attr in attrs {
    let ident = if let Some(ident) = attr.path().get_ident() {
      ident.to_string()
    } else {
      continue;
    };

    match ident.as_str() {
      "deprecated" => {
        deprecated = true;
      }
      "proto" => {
        attr.parse_nested_meta(|meta| {
          let ident = meta.path.require_ident()?.to_string();

          match ident.as_str() {
            "deprecated" => {
              let boolean = meta.parse_value::<LitBool>()?;

              deprecated = boolean.value();
            }
            "cel_rules" => {
              cel_rules.span = meta.input.span();
              cel_rules.set(
                meta
                  .parse_list::<PunctuatedItems<TokenStream2>>()?
                  .list,
              );
            }
            "reserved_names" => {
              let names = meta.parse_list::<StringList>()?;

              reserved_names = names.list;
            }
            "reserved_numbers" => {
              let numbers = meta.parse_list::<ReservedNumbers>()?;

              reserved_numbers = numbers;
            }
            "derive" => {
              let list = meta.parse_list::<MetaList>()?;

              shadow_derives = Some(list);
            }
            "parent_message" => {
              parent_message = Some(
                meta
                  .expr_value()?
                  .as_path()?
                  .require_ident()?
                  .clone(),
              );
            }
            "options" => {
              options.span = meta.input.span();
              options.set(meta.expr_value()?.into_token_stream());
            }
            "from_proto" => {
              from_proto = Some(meta.expr_value()?.as_path_or_closure()?);
            }
            "into_proto" => {
              into_proto = Some(meta.expr_value()?.as_path_or_closure()?);
            }
            "name" => {
              proto_name = Some(meta.parse_value::<ParsedStr>()?);
            }
            _ => return Err(meta.error("Unknown attribute")),
          };

          Ok(())
        })?;
      }
      _ => {}
    }
  }

  let name = proto_name
    .unwrap_or_else(|| ParsedStr::with_default_span(to_pascal_case(&struct_ident.to_string())));

  Ok(MessageAttrs {
    reserved_names,
    reserved_numbers,
    options,
    name,
    parent_message,
    from_proto,
    into_proto,
    shadow_derives,
    cel_rules,
    is_proxied: macro_args.is_proxied,
    no_auto_test: macro_args.no_auto_test,
    extern_path: macro_args.extern_path,
    deprecated,
  })
}
