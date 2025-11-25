use itertools::Itertools;
use syn::{GenericArgument, PathArguments, PathSegment};

use crate::*;

#[derive(Debug)]
pub enum ProtoTypes {
  String,
  Bool,
  Bytes,
  Enum(Path),
  Message,
  Int32,
}

impl ToTokens for ProtoTypes {
  fn to_tokens(&self, tokens: &mut TokenStream2) {
    let output = match self {
      ProtoTypes::String => quote! { string },
      ProtoTypes::Bool => quote! { bool },
      ProtoTypes::Bytes => quote! { bytes = "bytes" },
      ProtoTypes::Enum(path) => {
        let path_as_str = path.to_token_stream().to_string();

        quote! { enumeration = #path_as_str }
      }
      ProtoTypes::Message => quote! { message },
      ProtoTypes::Int32 => quote! { int32 },
    };

    tokens.extend(output)
  }
}

#[derive(Debug)]
pub enum ProtoTypeKind {
  Single(ProtoTypes),
  Repeated(ProtoTypes),
  Optional(ProtoTypes),
  Boxed,
  Map((ProtoTypes, ProtoTypes)),
}

impl ProtoTypeKind {
  pub fn is_option(&self) -> bool {
    matches!(self, Self::Optional(_))
  }
}

impl ToTokens for ProtoTypeKind {
  fn to_tokens(&self, tokens: &mut TokenStream2) {
    let output = match self {
      ProtoTypeKind::Single(inner) => inner.to_token_stream(),
      ProtoTypeKind::Repeated(inner) => quote! { #inner, repeated },
      ProtoTypeKind::Optional(inner) => quote! { #inner, optional },
      ProtoTypeKind::Boxed => quote! { message, optional, boxed },
      ProtoTypeKind::Map((k, v)) => {
        let k_as_str = k.to_token_stream().to_string();
        let v_as_str = v.to_token_stream().to_string();

        let map = format!("{k_as_str}, {v_as_str}");
        quote! { map = #map }
      }
    };

    tokens.extend(output)
  }
}

pub struct PathWrapper<'a> {
  pub inner: &'a Path,
}

impl<'a> PathWrapper<'a> {
  pub fn new(path: &'a Path) -> Self {
    Self { inner: path }
  }

  pub fn last_segment(&'_ self) -> PathSegmentWrapper<'_> {
    PathSegmentWrapper::new(self.inner.segments.last().unwrap())
  }
}

pub struct PathSegmentWrapper<'a> {
  pub inner: &'a PathSegment,
}

impl<'a> PathSegmentWrapper<'a> {
  pub fn new(segment: &'a PathSegment) -> Self {
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

pub fn get_proto_type(original_type: &Path) -> ProtoTypes {
  let last_segment = PathSegmentWrapper::new(original_type.segments.last().unwrap());
  let type_ident = last_segment.ident().to_string();

  match type_ident.as_str() {
    "String" => ProtoTypes::String,
    "bool" => ProtoTypes::Bool,
    "i32" => ProtoTypes::Int32,
    _ => ProtoTypes::Message,
  }
}

pub fn get_proto_type_outer(original_type: &Path) -> ProtoTypeKind {
  let path_wrapper = PathWrapper::new(original_type);

  let last_segment = path_wrapper.last_segment();

  let type_ident = last_segment.ident().to_string();

  match type_ident.as_str() {
    "Option" => {
      let inner = last_segment.first_argument().unwrap();

      ProtoTypeKind::Optional(get_proto_type(inner))
    }
    "Box" => ProtoTypeKind::Boxed,
    "Vec" | "ProtoRepeated" => {
      let inner = last_segment.first_argument().unwrap();

      ProtoTypeKind::Repeated(get_proto_type(inner))
    }
    "HashMap" | "ProtoMap" => {
      let (key, val) = last_segment.first_two_arguments().unwrap();

      ProtoTypeKind::Map((get_proto_type(key), get_proto_type(val)))
    }
    "ProtoEnum" => {
      let inner = last_segment.first_argument().unwrap();

      ProtoTypeKind::Single(ProtoTypes::Enum(inner.clone()))
    }
    _ => ProtoTypeKind::Single(get_proto_type(path_wrapper.inner)),
  }
}

#[derive(Debug)]
pub enum FieldTypeKind {
  Normal,
  Option,
  Boxed,
}

#[derive(Debug)]
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

pub fn extract_type_path(ty: &Type) -> Result<&Path, Error> {
  match ty {
    Type::Path(type_path) => Ok(&type_path.path),

    _ => Err(spanned_error!(ty, "Must be a type path")),
  }
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

pub fn extract_oneof_ident(ty: &Type) -> Result<Ident, Error> {
  let path = extract_type_path(ty)?;

  let path_wrapper = PathWrapper::new(path);
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
