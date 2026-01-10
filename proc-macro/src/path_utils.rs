use convert_case::{Boundary, Case, Casing};
use quote::format_ident;

use crate::*;

pub fn to_snake_case(str: &str) -> String {
  str
    .remove_boundaries(&[Boundary::UpperDigit, Boundary::LowerDigit])
    .to_case(Case::Snake)
}

pub fn to_upper_snake_case(str: &str) -> String {
  str
    .remove_boundaries(&[Boundary::UpperDigit, Boundary::LowerDigit])
    .to_case(Case::UpperSnake)
}

pub fn to_pascal_case(str: &str) -> String {
  str
    .remove_boundaries(&[Boundary::UpperDigit, Boundary::LowerDigit])
    .to_case(Case::Pascal)
}

#[derive(Default, Debug, Clone)]
pub enum ItemPathEntry {
  Path(Path),
  Proxied,
  #[default]
  None,
}

impl ItemPathEntry {
  pub fn get_path_or_fallback(&self, fallback: Option<&Path>) -> Option<Path> {
    let output = if let Self::Path(path) = self {
      path.clone()
    } else if let Some(fallback) = fallback {
      let fallback = fallback.clone();

      if matches!(self, Self::Proxied) {
        ident_with_proto_suffix(fallback)
      } else {
        fallback
      }
    } else {
      return None;
    };

    Some(output)
  }

  /// Returns `true` if the item path entry is [`Proxied`].
  ///
  /// [`Proxied`]: ItemPathEntry::Proxied
  #[must_use]
  pub const fn is_proxied(&self) -> bool {
    matches!(self, Self::Proxied)
  }
}

impl ToTokens for ItemPathEntry {
  fn to_tokens(&self, tokens: &mut TokenStream2) {
    match self {
      Self::Path(path) => path.to_tokens(tokens),
      _ => {}
    };
  }
}

pub fn ident_with_proto_suffix(mut path: Path) -> Path {
  let span = path.span();
  let last_segment = path.segments.last_mut().unwrap();

  last_segment.ident = format_ident!("{}Proto", last_segment.ident, span = span);

  path
}
