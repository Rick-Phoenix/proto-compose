use crate::*;

pub struct EnumAttrs {
  pub reserved_names: Vec<String>,
  pub reserved_numbers: ReservedNumbers,
  pub options: TokensOr<TokenStream2>,
  pub parent_message: Option<Ident>,
  pub name: String,
  pub no_prefix: bool,
  pub extern_path: Option<String>,
}

pub fn process_derive_enum_attrs(
  enum_ident: &Ident,
  attrs: &[Attribute],
) -> Result<EnumAttrs, Error> {
  let mut reserved_names: Vec<String> = Vec::new();
  let mut reserved_numbers = ReservedNumbers::default();
  let mut options = TokensOr::<TokenStream2>::new(|| quote! { vec![] });
  let mut proto_name: Option<String> = None;
  let mut no_prefix = false;
  let mut parent_message: Option<Ident> = None;
  let mut extern_path: Option<String> = None;

  parse_filtered_attrs(attrs, &["proto"], |meta| {
    let ident = meta.path.require_ident()?.to_string();

    match ident.as_str() {
      "reserved_names" => {
        let names = meta.parse_list::<StringList>()?;

        reserved_names = names.list;
      }
      "reserved_numbers" => {
        let numbers = meta.parse_list::<ReservedNumbers>()?;

        reserved_numbers = numbers;
      }
      "extern_path" => {
        extern_path = Some(meta.expr_value()?.as_string()?);
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
      "name" => {
        proto_name = Some(meta.expr_value()?.as_string()?);
      }
      "no_prefix" => no_prefix = true,
      _ => return Err(meta.error("Unknown attribute")),
    };

    Ok(())
  })?;

  let name = proto_name.unwrap_or_else(|| ccase!(pascal, enum_ident.to_string()));

  Ok(EnumAttrs {
    extern_path,
    reserved_names,
    reserved_numbers,
    options,
    name,
    no_prefix,
    parent_message,
  })
}
