use crate::*;

#[derive(Debug, Clone)]
pub struct OneofInfo {
  pub path: Path,
  pub tags: Vec<ParsedNum>,
  pub default: bool,
  pub required: bool,
}

pub fn tags_to_str(tags: &[ParsedNum]) -> String {
  let mut tags_str = String::new();

  for (i, tag) in tags.iter().enumerate() {
    tags_str.push_str(&tag.num.to_string());

    if i != tags.len() - 1 {
      tags_str.push_str(", ");
    }
  }

  tags_str
}

impl OneofInfo {
  pub fn parse(meta: &ParseNestedMeta, type_info: &TypeInfo) -> syn::Result<Self> {
    let mut oneof_path = ItemPathEntry::default();
    let mut tags: Vec<ParsedNum> = Vec::new();
    let mut default = false;
    let mut required = false;

    meta.parse_nested_meta(|meta| {
      let ident_str = meta.ident_str()?;

      match ident_str.as_str() {
        "default" => default = true,
        "proxied" => oneof_path = ItemPathEntry::Proxied,
        "required" => required = true,
        "tags" => {
          tags = meta
            .parse_list::<PunctuatedItems<ParsedNum>>()?
            .list;
        }
        _ => {
          oneof_path = ItemPathEntry::Path(meta.path);
        }
      };

      Ok(())
    })?;

    if default && required {
      return Err(meta.error("`default` and `required` cannot be used together"));
    }

    if tags.is_empty() {
      return Err(meta.error("Tags for oneofs must be set manually. You can set them with `#[proto(oneof(tags(1, 2, 3)))]`"));
    }

    let oneof_path = oneof_path
      .get_path_or_fallback(type_info.inner().as_path().as_ref())
      .ok_or_else(|| meta.error(
        "Failed to infer the path to the oneof. If this is a proxied oneof, use `oneof(proxied)`, otherwise set the path manually with `oneof(MyOneofPath)`"
    ))?;

    Ok(Self {
      path: oneof_path,
      tags,
      default,
      required,
    })
  }
}
