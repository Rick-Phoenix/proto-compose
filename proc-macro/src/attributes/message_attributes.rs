use syn_utils::PunctuatedItems;

use crate::*;

pub struct MessageAttrs {
  pub reserved_names: Vec<String>,
  pub reserved_numbers: ReservedNumbers,
  pub options: TokensOr<TokenStream2>,
  pub name: String,
  pub parent_message: Option<Ident>,
  pub from_proto: Option<PathOrClosure>,
  pub into_proto: Option<PathOrClosure>,
  pub shadow_derives: Option<MetaList>,
  pub cel_rules: IterTokensOr<TokenStream2>,
  pub is_proxied: bool,
  pub no_auto_test: bool,
  pub extern_path: Option<String>,
}

pub fn process_message_attrs(
  struct_ident: &Ident,
  macro_args: TokenStream2,
  attrs: &[Attribute],
) -> Result<MessageAttrs, Error> {
  let mut is_proxied = false;
  let mut no_auto_test = false;
  let mut extern_path: Option<String> = None;

  let parser = syn::meta::parser(|meta| {
    if let Some(ident) = meta.path.get_ident() {
      let ident = ident.to_string();

      match ident.as_str() {
        "proxied" => is_proxied = true,
        "no_auto_test" => no_auto_test = true,
        "extern_path" => extern_path = Some(meta.parse_value::<LitStr>()?.value()),
        _ => {}
      };
    }

    Ok(())
  });

  parser.parse2(macro_args)?;

  let mut reserved_names: Vec<String> = Vec::new();
  let mut reserved_numbers = ReservedNumbers::default();
  let mut options = TokensOr::<TokenStream2>::new(|| quote! { vec![] });
  let mut proto_name: Option<String> = None;
  let mut from_proto: Option<PathOrClosure> = None;
  let mut into_proto: Option<PathOrClosure> = None;
  let mut shadow_derives: Option<MetaList> = None;
  let mut cel_rules = IterTokensOr::<TokenStream2>::vec();
  let mut parent_message: Option<Ident> = None;

  parse_filtered_attrs(attrs, &["proto"], |meta| {
    let ident = meta.path.require_ident()?.to_string();

    match ident.as_str() {
      "cel_rules" => {
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
        options.set(meta.expr_value()?.into_token_stream());
      }
      "from_proto" => {
        from_proto = Some(meta.expr_value()?.as_path_or_closure()?);
      }
      "into_proto" => {
        into_proto = Some(meta.expr_value()?.as_path_or_closure()?);
      }
      "name" => {
        proto_name = Some(meta.expr_value()?.as_string()?);
      }
      _ => return Err(meta.error("Unknown attribute")),
    };

    Ok(())
  })?;

  let name = proto_name.unwrap_or_else(|| to_pascal_case(&struct_ident.to_string()));

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
    is_proxied,
    no_auto_test,
    extern_path,
  })
}
