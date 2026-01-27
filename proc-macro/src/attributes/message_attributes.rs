use syn::LitBool;

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
  pub proto_derives: Vec<Path>,
  pub forwarded_attrs: Vec<Meta>,
  pub is_proxied: bool,
  pub auto_tests: AutoTests,
  pub deprecated: bool,
  pub validators: Validators,
}

impl MessageAttrs {
  pub const fn has_custom_conversions(&self) -> bool {
    self.from_proto.is_some() && self.into_proto.is_some()
  }
}

#[derive(Default, Clone, Copy)]
pub struct MessageMacroArgs {
  pub is_proxied: bool,
}

impl MessageMacroArgs {
  pub fn parse(macro_args: TokenStream2) -> syn::Result<Self> {
    let mut is_proxied = false;

    let parser = syn::meta::parser(|meta| {
      if let Some(ident) = meta.path.get_ident() {
        let ident = ident.to_string();

        match ident.as_str() {
          "proxied" => is_proxied = true,
          _ => return Err(meta.error("Unknown attribute")),
        };
      }

      Ok(())
    });

    parser.parse2(macro_args)?;

    Ok(Self { is_proxied })
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
  let mut shadow_derives: Vec<Path> = Vec::new();
  let mut parent_message: Option<Ident> = None;
  let mut deprecated = false;
  let mut validators = Validators::default();
  let mut auto_tests = AutoTests::default();
  let mut forwarded_attrs: Vec<Meta> = Vec::new();

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
            "attr" => {
              forwarded_attrs = meta.parse_list::<PunctuatedItems<Meta>>()?.list;
            }
            "skip_checks" => {
              auto_tests = AutoTests::parse(&meta)?;
            }
            "validate" => {
              validators = meta.parse_value::<Validators>()?;
            }
            "deprecated" => {
              let boolean = meta.parse_value::<LitBool>()?;

              deprecated = boolean.value();
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
              shadow_derives = meta.parse_list::<PathList>()?.list;
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

  for validator in validators.validators.iter_mut() {
    if validator.kind.is_closure() {
      validator.expr = quote_spanned! {validator.span=>
        ::prelude::apply(::prelude::CelValidator::default(), #validator)
      }
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
    proto_derives: shadow_derives,
    is_proxied: macro_args.is_proxied,
    auto_tests,
    deprecated,
    validators,
    forwarded_attrs,
  })
}
