use crate::*;

#[derive(Clone, Debug)]
pub struct MessageInfo {
  pub path: Path,
  pub boxed: bool,
}

impl MessageInfo {
  pub fn parse(meta: &ParseNestedMeta, type_info: Option<&TypeInfo>) -> syn::Result<Self> {
    let mut item_path = ItemPathEntry::default();
    let mut boxed = false;

    // Checking first in case we just get `message` without the parentheses
    if meta.is_list() {
      meta.parse_nested_meta(|meta| {
        if let Ok(ident_str) = meta.ident_str() {
          match ident_str.as_str() {
            "proxied" => {
              item_path = ItemPathEntry::Proxied;
            }
            "boxed" => boxed = true,
            _ => item_path = ItemPathEntry::Path(meta.path),
          };
        } else {
          item_path = ItemPathEntry::Path(meta.path);
        }

        Ok(())
      })?;
    }

    let path = if let ItemPathEntry::Path(msg_path) = item_path {
      msg_path
    } else {
      // If type_info is None, it means the input was incorrect anyway (i.e. `repeated` without a Vec)
      // So the type we get at this point is already unnested by one degree
      let inferred_path = type_info
        .and_then(|type_info| match type_info.type_.as_ref() {
          // This still has to be checked because a message may not be marked as `optional`
          // So we might have to unnest the Option first
          RustType::Option(inner) => {
            if inner.is_box() {
              boxed = true;
              inner.inner().as_path()
            } else {
              inner.as_path()
            }
          }
          RustType::Box(inner) => {
            boxed = true;
            inner.as_path()
          }

          RustType::Other(type_path) => Some(type_path.path.clone()),
          _ => None,
        })
        .ok_or(meta.error("Failed to infer the message path. Please set it manually"))?;

      if item_path.is_proxied() {
        ident_with_proto_suffix(inferred_path)
      } else {
        inferred_path
      }
    };

    Ok(Self { path, boxed })
  }
}
