use quote::format_ident;

use crate::*;

pub fn append_proto_ident(mut path: Path) -> Path {
  let last_segment = path.segments.last_mut().unwrap();

  last_segment.ident = format_ident!("{}Proto", last_segment.ident);

  path
}

pub struct PathWrapper<'a> {
  pub inner: Cow<'a, Path>,
}

impl<'a> PathWrapper<'a> {
  pub fn new(path: Cow<'a, Path>) -> Self {
    Self { inner: path }
  }

  pub fn last_segment(&'_ self) -> PathSegmentWrapper<'_> {
    PathSegmentWrapper::new(Cow::Borrowed(self.inner.segments.last().unwrap()))
  }
}

pub struct PathSegmentWrapper<'a> {
  pub inner: Cow<'a, PathSegment>,
}

impl<'a> PathSegmentWrapper<'a> {
  pub fn new(segment: Cow<'a, PathSegment>) -> Self {
    Self { inner: segment }
  }

  pub fn ident(&self) -> &Ident {
    &self.inner.ident
  }

  pub fn get_arguments(&self) -> Option<impl Iterator<Item = &Path>> {
    if let PathArguments::AngleBracketed(args) = &self.inner.arguments {
      Some(args.args.iter().filter_map(|arg| {
        if let GenericArgument::Type(typ) = arg && let Type::Path(path) = typ {
        Some(&path.path)
      } else { None }
      }))
    } else {
      None
    }
  }

  pub fn first_argument(&self) -> Option<&Path> {
    self
      .get_arguments()
      .and_then(|args| args.find_or_first(|_| true))
  }

  pub fn first_two_arguments(&self) -> Option<(&Path, &Path)> {
    self.get_arguments().and_then(|args| {
      let mut first_arg: Option<&Path> = None;
      let mut second_arg: Option<&Path> = None;
      for (i, arg) in args.enumerate() {
        if i == 0 {
          first_arg = Some(arg);
        } else if i == 1 {
          second_arg = Some(arg);
          break;
        }
      }

      if let Some(first) = first_arg && let Some(second) = second_arg {
          Some((first, second))
        } else {
          None
        }
    })
  }
}

pub fn extract_type_path(ty: &Type) -> Result<&Path, Error> {
  match ty {
    Type::Path(type_path) => Ok(&type_path.path),

    _ => Err(spanned_error!(ty, "Must be a type path")),
  }
}

pub fn extract_type_path_mut(ty: &mut Type) -> Result<&mut Path, Error> {
  match ty {
    Type::Path(type_path) => Ok(&mut type_path.path),

    _ => Err(spanned_error!(ty, "Must be a type path")),
  }
}

pub fn extract_oneof_ident(ty: &Type) -> Result<Ident, Error> {
  let path = extract_type_path(ty)?;

  let path_wrapper = PathWrapper::new(Cow::Borrowed(path));
  let last_segment = path_wrapper.last_segment();

  if last_segment.ident() != "Option" {
    return Err(spanned_error!(ty, "Oneofs must be wrapped in Option"));
  }

  last_segment
    .first_argument()
    .ok_or(spanned_error!(ty, "Could not find argument to Option"))?
    .require_ident()
    .cloned()
}
