use crate::*;

#[derive(Clone)]
pub enum RustType {
  Option(Path),
  OptionBoxed(Path),
  Boxed(Path),
  Map((Path, Path)),
  Vec(Path),
  Normal(Path),
}

impl RustType {
  pub fn as_inner_option_path(&self) -> Option<&Path> {
    if let RustType::Option(path) = self {
      Some(path)
    } else {
      None
    }
  }

  pub fn inner_path(&self) -> Option<&Path> {
    let output = match self {
      RustType::Option(path) => path,
      RustType::OptionBoxed(path) => path,
      RustType::Map(_) => return None,
      RustType::Vec(path) => path,
      RustType::Normal(path) => path,
      RustType::Boxed(path) => path,
    };

    Some(output)
  }

  pub fn as_map(&self) -> Option<&(Path, Path)> {
    if let Self::Map(v) = self {
      Some(v)
    } else {
      None
    }
  }

  /// Returns `true` if the rust type is [`Option`].
  ///
  /// [`Option`]: RustType::Option
  #[must_use]
  pub fn is_option(&self) -> bool {
    matches!(self, Self::Option(..) | Self::OptionBoxed(..))
  }
}

impl RustType {
  pub fn from_type(ty: &Type, item_ident: &Ident) -> Result<Self, Error> {
    let path = extract_type_path(ty)?;

    Ok(Self::from_path(path, item_ident))
  }

  pub fn from_path(path: &Path, item_ident: &Ident) -> Self {
    let path_wrapper = PathWrapper::new(Cow::Borrowed(path));

    let last_segment = path_wrapper.last_segment();

    let type_ident = last_segment.ident().to_string();

    match type_ident.as_str() {
      "Box" => {
        let inner = PathWrapper::new(Cow::Borrowed(last_segment.first_argument().unwrap()));

        Self::Boxed(inner.inner.into_owned())
      }
      "Option" => {
        let inner = PathWrapper::new(Cow::Borrowed(last_segment.first_argument().unwrap()));

        let inner_last_segment = inner.last_segment();

        if inner_last_segment.ident() == "Box" {
          let box_wrapper = PathWrapper::new(Cow::Borrowed(&inner.inner));

          let last_segment = box_wrapper.last_segment();

          let box_inner = last_segment.first_argument().unwrap();

          if let Some(boxed_item_ident) = box_inner.get_ident() && boxed_item_ident == item_ident {
            Self::OptionBoxed(box_inner.clone())
          } else {
            Self::Option(inner.inner.into_owned())
          }
        } else {
          Self::Option(inner.inner.into_owned())
        }
      }
      "Vec" => {
        let inner = last_segment.first_argument().unwrap();

        Self::Vec(inner.clone())
      }
      "HashMap" => {
        let (key, val) = last_segment.first_two_arguments().unwrap();

        Self::Map((key.clone(), val.clone()))
      }
      _ => Self::Normal(path.clone()),
    }
  }
}
