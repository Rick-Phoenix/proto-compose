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
  pub cel_rules: Option<Vec<Expr>>,
  pub is_direct: bool,
  pub no_auto_test: bool,
  pub extern_path: Option<String>,
}

pub fn process_derive_message_attrs(
  rust_name: &Ident,
  macro_attrs: MessageMacroAttrs,
  attrs: &[Attribute],
) -> Result<MessageAttrs, Error> {
  let mut reserved_names: Vec<String> = Vec::new();
  let mut reserved_numbers = ReservedNumbers::default();
  let mut options = TokensOr::<TokenStream2>::new(|| quote! { vec![] });
  let mut proto_name: Option<String> = None;
  let mut from_proto: Option<PathOrClosure> = None;
  let mut into_proto: Option<PathOrClosure> = None;
  let mut shadow_derives: Option<MetaList> = None;
  let mut cel_rules: Option<Vec<Expr>> = None;
  let mut parent_message: Option<Ident> = None;

  parse_filtered_attrs(attrs, &["proto"], |meta| {
    let ident = meta.path.require_ident()?.to_string();

    match ident.as_str() {
      "cel_rules" => {
        cel_rules = Some(meta.parse_list::<PunctuatedItems<Expr>>()?.list);
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
      "direct" => {
        return Err(
          meta.error("`direct` must be set as a proc macro argument, not as an attribute"),
        );
      }
      _ => return Err(meta.error("Unknown attribute")),
    };

    Ok(())
  })?;

  let name = proto_name.unwrap_or_else(|| ccase!(pascal, rust_name.to_string()));

  Ok(MessageAttrs {
    reserved_names,
    reserved_numbers,
    options,
    name,
    from_proto,
    into_proto,
    shadow_derives,
    cel_rules,
    is_direct: macro_attrs.is_direct,
    no_auto_test: macro_attrs.no_auto_test,
    extern_path: macro_attrs.extern_path,
    parent_message,
  })
}
