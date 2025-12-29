use syn_utils::parse_comma_separated;

use crate::*;

#[derive(Debug, Clone, Default)]
pub struct OneofInfo {
  pub path: ItemPathEntry,
  pub tags: Vec<i32>,
  pub default: bool,
  pub required: bool,
}

pub fn tags_to_str(tags: &[i32]) -> String {
  let mut tags_str = String::new();

  for (i, tag) in tags.iter().enumerate() {
    tags_str.push_str(&tag.to_string());

    if i != tags.len() - 1 {
      tags_str.push_str(", ");
    }
  }

  tags_str
}

impl Parse for OneofInfo {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let mut oneof_path = ItemPathEntry::default();
    let mut tags: Vec<i32> = Vec::new();
    let mut default = false;
    let mut required = false;
    let input_span = input.span();

    parse_comma_separated(input, |meta| {
      match meta {
        Meta::Path(path) => {
          if let Some(ident) = path.get_ident() {
            let ident_str = ident.to_string();

            match ident_str.as_str() {
              "default" => default = true,
              "proxied" => oneof_path = ItemPathEntry::Proxied,
              "required" => required = true,
              _ => {}
            };
          } else {
            oneof_path = ItemPathEntry::Path(path);
          }
        }
        Meta::List(list) => {
          let ident = list.path.require_ident()?;
          if ident == "tags" {
            tags = list.parse_args::<NumList>()?.list;
          }
        }
        _ => {}
      };

      Ok(())
    })?;

    if default && required {
      bail_with_span!(
        input_span,
        "`default` and `required` cannot be used together"
      );
    }

    Ok(Self {
      path: oneof_path,
      tags,
      default,
      required,
    })
  }
}
