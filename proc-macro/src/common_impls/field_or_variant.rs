use crate::*;

pub enum FieldOrVariant<'a> {
  Field(&'a mut Field),
  Variant(&'a mut Variant),
}

impl<'a> FieldOrVariant<'a> {
  pub fn ident(&self) -> syn::Result<&Ident> {
    match self {
      FieldOrVariant::Field(field) => field.require_ident(),
      FieldOrVariant::Variant(variant) => Ok(&variant.ident),
    }
  }

  pub fn get_type(&self) -> syn::Result<&Type> {
    let output = match self {
      FieldOrVariant::Field(field) => &field.ty,
      FieldOrVariant::Variant(variant) => {
        if let Fields::Unnamed(variant_fields) = &variant.fields
          && variant_fields.unnamed.len() == 1
        {
          &variant_fields.unnamed.first().unwrap().ty
        } else {
          bail!(
            &variant.fields,
            "Oneof variants must contain a single unnamed value"
          );
        }
      }
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
      FieldOrVariant::Variant(variant) => {
        if let Fields::Unnamed(variant_fields) = &mut variant.fields {
          if variant_fields.unnamed.len() != 1 {
            bail!(
              &variant.fields,
              "Oneof variants must contain a single unnamed value"
            );
          }

          &mut variant_fields.unnamed.first_mut().unwrap().ty
        } else {
          bail!(
            &variant.fields,
            "Oneof variants must contain a single unnamed value"
          );
        }
      }
    };

    *src_type = ty;

    Ok(())
  }
}
