use syn::spanned::Spanned;

use crate::*;

#[derive(Default, Debug, Clone)]
pub enum ProtoFieldKind {
  Message(MessageInfo),
  Enum(Option<Path>),
  Oneof(OneofInfo),
  Map(ProtoMap),
  Repeated(Box<ProtoFieldKind>),
  Sint32,
  #[default]
  None,
}

impl ProtoFieldKind {
  pub fn supports_repeated(&self) -> bool {
    !matches!(self, Self::Oneof(_) | Self::Repeated(_) | Self::Map(_))
  }

  pub fn parse_repeated(list: MetaList) -> Result<Self, Error> {
    let arg = list.parse_args::<Meta>()?;

    let error_msg = "Unrecognized repeated field kind";

    let output = match arg {
      Meta::Path(path) => {
        let ident_str = path.require_ident()?.to_string();

        let inner = Self::from_str(&ident_str).ok_or(spanned_error!(&path, error_msg))?;

        Self::Repeated(Box::new(inner))
      }
      Meta::List(list) => {
        let list_name = list.path.require_ident()?.to_string();
        let span = list.span();

        let inner = Self::from_meta_list(&list_name, list)?.ok_or(error!(span, error_msg))?;

        if !inner.supports_repeated() {
          return Err(error!(
            span,
            "This type is not supported for repeated fields"
          ));
        }

        Self::Repeated(Box::new(inner))
      }
      Meta::NameValue(_) => return Err(spanned_error!(arg, "Expected a MetaList or Path")),
    };

    Ok(output)
  }

  pub fn from_meta_list(list_name: &str, list: MetaList) -> Result<Option<Self>, Error> {
    let output = match list_name {
      "message" => {
        let message_info = list.parse_args::<MessageInfo>()?;

        Self::Message(message_info)
      }
      "enum_" => {
        let enum_path = list.parse_args::<Path>()?;

        Self::Enum(Some(enum_path))
      }
      "map" => {
        let map_data = list.parse_args::<ProtoMap>()?;

        Self::Map(map_data)
      }

      _ => return Ok(None),
    };

    Ok(Some(output))
  }

  pub fn from_str(str: &str) -> Option<Self> {
    let output = match str {
      "oneof" => Self::Oneof(OneofInfo::default()),
      "enum_" => Self::Enum(None),
      "message" => Self::Message(MessageInfo::default()),
      "sint32" => Self::Sint32,

      _ => return None,
    };

    Some(output)
  }
}

impl ProtoFieldKind {
  pub fn is_oneof(&self) -> bool {
    matches!(self, Self::Oneof { .. })
  }
}
