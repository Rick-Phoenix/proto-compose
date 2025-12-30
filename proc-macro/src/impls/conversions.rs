use crate::*;

pub enum FieldConversionKind<'a> {
  StructField {
    ident: &'a Ident,
  },
  EnumVariant {
    variant_ident: &'a Ident,
    source_enum_ident: &'a Ident,
    target_enum_ident: &'a Ident,
  },
}

impl<'a> FieldConversionKind<'a> {
  pub fn base_ident(&self) -> TokenStream2 {
    match self {
      Self::StructField { ident } => quote! { value.#ident },
      // With enums, we always pattern match first so we always
      // have the same ident to process
      Self::EnumVariant { .. } => quote! { v },
    }
  }

  pub fn conversion_from_source_to_target(&self, conversion_expr: &TokenStream2) -> TokenStream2 {
    match self {
      Self::StructField { ident } => quote! {
        #ident: #conversion_expr,
      },
      Self::EnumVariant {
        variant_ident,
        source_enum_ident,
        target_enum_ident,
      } => quote! {
        #source_enum_ident::#variant_ident(v) => #target_enum_ident::#variant_ident(#conversion_expr),
      },
    }
  }

  pub fn conversion_from_target_to_source(&self, conversion_expr: &TokenStream2) -> TokenStream2 {
    match self {
      Self::StructField { ident } => quote! {
        #ident: #conversion_expr,
      },
      Self::EnumVariant {
        variant_ident,
        source_enum_ident,
        target_enum_ident,
      } => quote! {
        #target_enum_ident::#variant_ident(v) => #source_enum_ident::#variant_ident(#conversion_expr),
      },
    }
  }
}

fn process_custom_expression(expr: &PathOrClosure, base_ident: &TokenStream2) -> TokenStream2 {
  match expr {
    PathOrClosure::Path(path) => quote! { #path(#base_ident) },
    PathOrClosure::Closure(closure) => {
      quote! {
        ::prelude::apply(#base_ident, #closure)
      }
    }
  }
}

pub struct FromImpl<'a> {
  pub source_ident: &'a Ident,
  pub target_ident: &'a Ident,
  pub kind: InputItemKind,
  pub conversion_data: &'a ConversionData<'a>,
}

pub struct ProtoConversionImpl<'a> {
  pub source_ident: &'a Ident,
  pub target_ident: &'a Ident,
  pub kind: InputItemKind,
  pub into_proto: ConversionData<'a>,
  pub from_proto: ConversionData<'a>,
}

impl<'a> ProtoConversionImpl<'a> {
  pub fn generate_conversion_impls(&self) -> TokenStream2 {
    let mut tokens = TokenStream2::new();

    tokens.extend(self.create_into_proto_impl());
    tokens.extend(self.create_from_proto_impl());
    tokens.extend(self.create_conversion_helpers());

    tokens
  }

  pub fn create_into_proto_impl(&self) -> TokenStream2 {
    let Self {
      source_ident,
      target_ident,
      kind,
      into_proto,
      ..
    } = self;

    create_from_impl(&FromImpl {
      source_ident,
      target_ident,
      kind: *kind,
      conversion_data: into_proto,
    })
  }

  // This one only really makes sense for messages, not for oneofs
  pub fn create_validated_conversion_helpers(&self) -> TokenStream2 {
    let Self {
      source_ident,
      target_ident,
      ..
    } = self;

    quote! {
      impl #source_ident {
        pub fn from_validated_proto(value: #target_ident) -> Result<Self, Violations> {
          match value.validate() {
            Ok(_) => Ok(value.into()),
            Err(vi) => Err(vi)
          }
        }

        pub fn into_validated_proto(self) -> Result<#target_ident, Violations> {
          let output: #target_ident = self.into();

          match output.validate() {
            Ok(_) => Ok(output),
            Err(vi) => Err(vi)
          }
        }
      }
    }
  }

  pub fn create_conversion_helpers(&self) -> TokenStream2 {
    let Self {
      source_ident,
      target_ident,
      ..
    } = self;

    quote! {
      impl #source_ident {
        pub fn from_proto(value: #target_ident) -> Self {
          value.into()
        }

        pub fn into_proto(self) -> #target_ident {
          self.into()
        }
      }
    }
  }

  pub fn create_from_proto_impl(&self) -> TokenStream2 {
    let Self {
      source_ident,
      target_ident,
      kind,
      from_proto,
      ..
    } = self;

    let switched = FromImpl {
      // Note: source and target are switched here
      source_ident: target_ident,
      target_ident: source_ident,
      kind: *kind,
      conversion_data: from_proto,
    };

    create_from_impl(&switched)
  }

  pub fn handle_field_conversions(&mut self, field_attr_data: &FieldDataKind) {
    match field_attr_data {
      FieldDataKind::Ignored { from_proto, ident } => {
        if !self.from_proto.has_custom_impl() {
          self.add_field_from_proto_impl(from_proto, None, ident);
        }
      }
      FieldDataKind::Normal(field_attrs) => {
        if !self.from_proto.has_custom_impl() {
          self.add_field_from_proto_impl(
            &field_attrs.from_proto,
            Some(&field_attrs.proto_field),
            &field_attrs.ident,
          );
        }

        if !self.into_proto.has_custom_impl() {
          self.add_field_into_proto_impl(
            &field_attrs.into_proto,
            &field_attrs.proto_field,
            &field_attrs.ident,
          );
        }
      }
    }
  }

  pub fn add_field_into_proto_impl(
    &mut self,
    custom_expression: &Option<PathOrClosure>,
    proto_field: &ProtoField,
    field_ident: &Ident,
  ) {
    let field_conversion_kind = match &self.kind {
      InputItemKind::Enum => FieldConversionKind::EnumVariant {
        variant_ident: field_ident,
        source_enum_ident: self.source_ident,
        target_enum_ident: self.target_ident,
      },
      InputItemKind::Struct => FieldConversionKind::StructField { ident: field_ident },
    };

    let base_ident = field_conversion_kind.base_ident();

    let conversion_expr = if let Some(expr) = custom_expression {
      process_custom_expression(expr, &base_ident)
    } else if let ProtoField::Oneof(OneofInfo { default: true, .. }) = &proto_field {
      quote! { Some(#base_ident.into()) }
    } else {
      proto_field.default_into_proto(&base_ident)
    };

    let conversion = field_conversion_kind.conversion_from_source_to_target(&conversion_expr);

    self.into_proto.tokens.extend(conversion);
  }

  pub fn add_field_from_proto_impl(
    &mut self,
    custom_expression: &Option<PathOrClosure>,
    proto_field: Option<&ProtoField>,
    field_ident: &Ident,
  ) {
    let field_conversion_kind = match &self.kind {
      InputItemKind::Enum => FieldConversionKind::EnumVariant {
        variant_ident: field_ident,
        source_enum_ident: self.source_ident,
        target_enum_ident: self.target_ident,
      },
      InputItemKind::Struct => FieldConversionKind::StructField { ident: field_ident },
    };

    let conversion_expr = if let Some(proto_field) = proto_field {
      let base_ident = field_conversion_kind.base_ident();

      if let Some(expr) = custom_expression {
        process_custom_expression(expr, &base_ident)
      } else {
        proto_field.default_from_proto(&base_ident)
      }
    } else {
      if let Some(expr) = custom_expression {
        match expr {
          // Field is ignored, so we don't pass any args here
          PathOrClosure::Path(path) => quote! { #path() },
          PathOrClosure::Closure(closure) => {
            let error = error!(closure, "Cannot use a closure for ignored fields");

            error.into_compile_error()
          }
        }
      } else {
        quote! { Default::default() }
      }
    };

    let conversion = field_conversion_kind.conversion_from_target_to_source(&conversion_expr);

    self.from_proto.tokens.extend(conversion);
  }
}

// This is used as a wrapper to store the custom expression that was given
// (if there was one) or provide the implementation tokens (if there wasn't one)
pub struct ConversionData<'a> {
  pub custom_expression: &'a Option<PathOrClosure>,
  pub tokens: TokenStream2,
}

impl<'a> ConversionData<'a> {
  pub fn has_custom_impl(&self) -> bool {
    self.custom_expression.is_some()
  }

  pub fn new(custom_expression: &'a Option<PathOrClosure>) -> Self {
    Self {
      custom_expression,
      tokens: TokenStream2::new(),
    }
  }
}

#[derive(Clone, Copy)]
pub enum InputItemKind {
  Enum,
  Struct,
}

fn create_from_impl(info: &FromImpl) -> TokenStream2 {
  let FromImpl {
    source_ident,
    target_ident,
    kind,
    conversion_data:
      ConversionData {
        custom_expression,
        tokens: conversion_tokens,
      },
  } = info;

  let conversion_body = if let Some(expr) = custom_expression {
    let base_ident = quote! { value };

    process_custom_expression(expr, &base_ident)
  } else {
    match kind {
      InputItemKind::Enum => quote! {
        match value {
          #conversion_tokens
        }
      },
      InputItemKind::Struct => quote! {
        Self {
          #conversion_tokens
        }
      },
    }
  };

  quote! {
    impl From<#source_ident> for #target_ident {
      fn from(value: #source_ident) -> Self {
        #conversion_body
      }
    }
  }
}
