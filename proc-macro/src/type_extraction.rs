use syn::{GenericArgument, PathArguments, PathSegment};

use crate::*;

pub enum FieldTypeKind {
  Normal,
  Option,
  Boxed,
}

pub struct FieldType {
  pub outer: Path,
  pub inner: Option<Path>,
  pub kind: FieldTypeKind,
}

impl FieldType {
  pub fn inner(&self) -> &Path {
    self.inner.as_ref().unwrap_or(&self.outer)
  }

  pub fn is_option(&self) -> bool {
    matches!(self.kind, FieldTypeKind::Option)
  }
  pub fn is_boxed(&self) -> bool {
    matches!(self.kind, FieldTypeKind::Boxed)
  }
}

fn extract_inner_type(path_segment: &PathSegment) -> Option<Path> {
  if let PathArguments::AngleBracketed(args) = &path_segment.arguments
    && let GenericArgument::Type(inner_ty) = args.args.first()? && let Type::Path(type_path) = inner_ty {
      return Some(type_path.path.clone());
    }

  None
}

pub fn extract_type(ty: &Type) -> Result<FieldType, Error> {
  let outer = match ty {
    Type::Path(type_path) => type_path.path.clone(),

    _ => return Err(spanned_error!(ty, "Must be a type path")),
  };

  let last_segment = outer.segments.last().unwrap();

  let (inner, kind) = if last_segment.ident == "Option" {
    (
      Some(extract_inner_type(last_segment).unwrap()),
      FieldTypeKind::Option,
    )
  } else if last_segment.ident == "Box" {
    (
      Some(extract_inner_type(last_segment).unwrap()),
      FieldTypeKind::Boxed,
    )
  } else {
    (None, FieldTypeKind::Normal)
  };

  Ok(FieldType { outer, inner, kind })
}
