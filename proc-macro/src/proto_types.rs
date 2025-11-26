use std::str::FromStr;

use crate::*;

#[derive(Debug, Clone)]
pub enum ProtoType {
  String,
  Bool,
  Bytes,
  Enum(Path),
  Message,
  Int32,
  Map(Box<ProtoMap>),
}

impl ProtoType {
  pub fn validator_expr(&self, validator: &ValidatorExpr) -> TokenStream2 {
    let target_type = match self {
      ProtoType::String => quote! { String },
      ProtoType::Bool => quote! { bool },
      ProtoType::Bytes => quote! { Vec<u8> },
      ProtoType::Enum(_) => quote! { GenericProtoEnum },
      ProtoType::Message => quote! { GenericMessage },
      ProtoType::Int32 => quote! { i32 },
      ProtoType::Map(_) => todo!(),
      _ => todo!(),
    };

    match validator {
      ValidatorExpr::Call(call) => {
        quote! { Some(<ValidatorMap as ProtoValidator<#target_type>>::from_builder(#call)) }
      }

      ValidatorExpr::Closure(closure) => {
        quote! { Some(<ValidatorMap as ProtoValidator<#target_type>>::build_rules(#closure)) }
      }
    }
  }

  pub fn from_rust_type(type_info: &TypeInfo) -> Result<Self, Error> {
    let path = match &type_info.rust_type {
      RustType::Option(path) => path,
      RustType::Boxed(path) => path,
      RustType::Vec(path) => path,
      RustType::Normal(path) => path,
      RustType::Map((k, v)) => {
        let keys_str = k.require_ident()?.to_string();
        let values_str = v.require_ident()?.to_string();

        let keys = ProtoMapKeys::from_str(&keys_str).unwrap();
        let values = if values_str == "GenericProtoEnum" {
          ProtoMapValues::Enum(v.clone())
        } else {
          ProtoMapValues::from_str(&values_str)
            .map_err(|e| spanned_error!(&type_info.full_type, e))?
        };

        return Ok(ProtoType::Map(Box::new(ProtoMap { keys, values })));
      }
    };

    let last_segment = PathSegmentWrapper::new(Cow::Borrowed(path.segments.last().unwrap()));
    let type_ident = last_segment.ident().to_string();

    let output = match type_ident.as_str() {
      "String" => Self::String,
      "bool" => Self::Bool,
      "GenericMessage" => Self::Message,
      "i32" => Self::Int32,
      _ => {
        return Err(spanned_error!(
          path,
          format!("Type {type_ident} not recognized")
        ))
      }
    };

    Ok(output)
  }
}

impl ToTokens for ProtoType {
  fn to_tokens(&self, tokens: &mut TokenStream2) {
    let output = match self {
      ProtoType::String => quote! { string },
      ProtoType::Bool => quote! { bool },
      ProtoType::Bytes => quote! { bytes = "bytes" },
      ProtoType::Enum(path) => {
        let path_as_str = path.to_token_stream().to_string();

        quote! { enumeration = #path_as_str }
      }
      ProtoType::Message => quote! { message },
      ProtoType::Int32 => quote! { int32 },
      ProtoType::Map(map) => map.to_token_stream(),
    };

    tokens.extend(output)
  }
}
