use crate::*;

#[derive(Clone)]
pub enum RustType {
  Option(Path),
  Boxed(Path),
  Map((Path, Path)),
  Vec(Path),
  Normal(Path),
}

impl RustType {
  pub fn as_option(&self) -> Option<&Path> {
    if let Self::Option(path) = self {
      Some(path)
    } else {
      None
    }
  }

  pub fn inner_path(&self) -> Option<&Path> {
    let output = match self {
      RustType::Option(path) => path,
      RustType::Boxed(path) => path,
      RustType::Map(_) => return None,
      RustType::Vec(path) => path,
      RustType::Normal(path) => path,
    };

    Some(output)
  }
}

impl RustType {
  pub fn from_path(path: &Path) -> Self {
    let path_wrapper = PathWrapper::new(Cow::Borrowed(path));

    let last_segment = path_wrapper.last_segment();

    let type_ident = last_segment.ident().to_string();

    match type_ident.as_str() {
      "Option" => {
        let inner = PathWrapper::new(Cow::Borrowed(last_segment.first_argument().unwrap()));

        let inner_last_segment = inner.last_segment();

        if inner_last_segment.ident() == "Box" {
          let box_wrapper = PathWrapper::new(Cow::Borrowed(&inner.inner));

          let last_segment = box_wrapper.last_segment();

          let box_inner = last_segment.first_argument().unwrap();

          Self::Boxed(box_inner.clone())
        } else {
          Self::Option(inner.inner.into_owned())
        }
      }
      "Vec" | "ProtoRepeated" => {
        let inner = last_segment.first_argument().unwrap();

        Self::Vec(inner.clone())
      }
      "HashMap" | "ProtoMap" => {
        let (key, val) = last_segment.first_two_arguments().unwrap();

        Self::Map((key.clone(), val.clone()))
      }
      _ => Self::Normal(path.clone()),
    }
  }
}
