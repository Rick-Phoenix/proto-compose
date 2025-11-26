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
  pub fn from_rust_type(rust_type: &RustType) -> Result<Self, Error> {
    let path = match rust_type {
      RustType::Option(path) => path,
      RustType::Boxed(path) => path,
      RustType::Vec(path) => path,
      RustType::Normal(path) => path,
      RustType::Map(_) => panic!("Map not supported"),
    };

    let last_segment = PathSegmentWrapper::new(Cow::Borrowed(path.segments.last().unwrap()));
    let type_ident = last_segment.ident().to_string();

    let output = match type_ident.as_str() {
      "String" => Self::String,
      "bool" => Self::Bool,
      "ProtoMessage" => Self::Message,
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
