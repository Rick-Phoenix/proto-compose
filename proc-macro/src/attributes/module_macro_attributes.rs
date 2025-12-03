use crate::*;

pub struct ModuleAttrs {
  pub file: String,
  pub package: String,
  pub schema_feature: Option<String>,
}

impl ModuleAttrs {
  pub fn as_attribute(&self) -> Attribute {
    let Self {
      schema_feature,
      package,
      file,
    } = self;

    let schema_feature_tokens = if let Some(feature) = schema_feature {
      Some(quote! { , schema_feature = #feature })
    } else {
      None
    };

    parse_quote! { #[proto(file = #file, package = #package #schema_feature_tokens)] }
  }
}

impl Parse for ModuleAttrs {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let mut file: Option<String> = None;
    let mut package: Option<String> = None;
    let mut schema_feature: Option<String> = None;

    let args = Punctuated::<MetaNameValue, Token![,]>::parse_terminated(input)?;

    for arg in args {
      let ident = get_ident_or_continue!(arg.path);

      match ident.as_str() {
        "file" => {
          file = Some(extract_string_lit(&arg.value)?);
        }
        "package" => {
          package = Some(extract_string_lit(&arg.value)?);
        }
        "schema_feature" => {
          schema_feature = Some(extract_string_lit(&arg.value)?);
        }
        _ => {}
      };
    }

    let file = file.ok_or(error!(Span::call_site(), "File attribute is missing"))?;
    let package = package.ok_or(error!(Span::call_site(), "Package attribute is missing"))?;

    Ok(ModuleAttrs {
      file,
      package,
      schema_feature,
    })
  }
}
