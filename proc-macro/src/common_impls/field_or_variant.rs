use syn_utils::EnumVariant;

use crate::*;

pub enum FieldOrVariant<'a> {
  Field(&'a mut Field),
  Variant(&'a mut Variant),
}

impl<'a> From<&'a mut Field> for FieldOrVariant<'a> {
  fn from(value: &'a mut Field) -> Self {
    Self::Field(value)
  }
}

impl<'a> From<&'a mut Variant> for FieldOrVariant<'a> {
  fn from(value: &'a mut Variant) -> Self {
    Self::Variant(value)
  }
}

impl<'a> FieldOrVariant<'a> {
  pub fn attributes(&self) -> &[Attribute] {
    match self {
      FieldOrVariant::Field(field) => &field.attrs,
      FieldOrVariant::Variant(variant) => &variant.attrs,
    }
  }

  pub fn span(&self) -> Span {
    match self {
      FieldOrVariant::Field(field) => field.span(),
      FieldOrVariant::Variant(variant) => variant.span(),
    }
  }

  pub fn ident(&self) -> syn::Result<&Ident> {
    match self {
      FieldOrVariant::Field(field) => field.require_ident(),
      FieldOrVariant::Variant(variant) => Ok(&variant.ident),
    }
  }

  pub fn get_type(&self) -> syn::Result<&Type> {
    let output = match self {
      FieldOrVariant::Field(field) => &field.ty,
      FieldOrVariant::Variant(variant) => variant.type_()?,
    };

    Ok(output)
  }

  pub fn inject_attr(&mut self, attr: Attribute) {
    match self {
      FieldOrVariant::Field(field) => field.attrs.push(attr),
      FieldOrVariant::Variant(variant) => variant.attrs.push(attr),
    }
  }

  pub fn change_type(&mut self, ty: Type) -> Result<(), Error> {
    let src_type = match self {
      FieldOrVariant::Field(field) => &mut field.ty,
      FieldOrVariant::Variant(variant) => variant.type_mut()?,
    };

    *src_type = ty;

    Ok(())
  }

  /// Returns `true` if the field or variant is [`Variant`].
  ///
  /// [`Variant`]: FieldOrVariant::Variant
  #[must_use]
  pub fn is_variant(&self) -> bool {
    matches!(self, Self::Variant(..))
  }
}
