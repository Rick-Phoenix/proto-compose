use crate::*;

impl FieldData {
  pub fn descriptor_type_tokens(&self) -> TokenStream2 {
    match &self.proto_field {
      ProtoField::Map(_) => {
        quote_spanned! {self.span=> ::prelude::proto_types::field_descriptor_proto::Type::Message }
      }
      ProtoField::Repeated(inner) | ProtoField::Optional(inner) | ProtoField::Single(inner) => {
        inner.descriptor_type_tokens(self.span)
      }
      ProtoField::Oneof { .. } => {
        quote_spanned! {self.span=> compile_error!("Validator tokens should not be triggered for a oneof field") }
      }
    }
  }

  pub fn as_prost_attr(&self) -> Attribute {
    let inner = match &self.proto_field {
      ProtoField::Oneof(OneofInfo { path, tags, .. }) => {
        let oneof_path_str = path.to_token_stream().to_string();
        let tags_str = tags_to_str(tags);

        // We don't need to add the tag for oneofs,
        // so we return early
        return parse_quote_spanned! {self.span=> #[prost(oneof = #oneof_path_str, tags = #tags_str)] };
      }
      ProtoField::Map(map) => {
        let map_attr = format!("{}, {}", map.keys, map.values.as_prost_map_value());

        quote_spanned! {self.span=> map = #map_attr }
      }

      ProtoField::Repeated(proto_type) => {
        let p_type = proto_type.as_prost_attr_type(self.span);

        quote_spanned! {self.span=> #p_type, repeated }
      }
      ProtoField::Optional(proto_type) => {
        let p_type = proto_type.as_prost_attr_type(self.span);

        quote_spanned! {self.span=> #p_type, optional }
      }
      ProtoField::Single(proto_type) => proto_type.as_prost_attr_type(self.span),
    };

    let tag_as_str = self
      .tag
      .as_ref()
      .map(|t| t.num)
      .unwrap_or_default()
      .to_string();

    parse_quote_spanned! {self.span=> #[prost(#inner, tag = #tag_as_str)] }
  }

  pub fn validator_name(&self) -> TokenStream2 {
    match &self.proto_field {
      ProtoField::Map(map) => {
        let keys = map
          .keys
          .into_type()
          .validator_target_type(self.span);
        let values = map.values.validator_target_type(self.span);

        quote_spanned! {self.span=> MapValidator<#keys, #values> }
      }
      ProtoField::Oneof { .. } => quote! {},
      ProtoField::Repeated(proto_type) => {
        let inner = proto_type.validator_target_type(self.span);

        quote_spanned! {self.span=> RepeatedValidator<#inner> }
      }
      ProtoField::Optional(proto_type) | ProtoField::Single(proto_type) => {
        proto_type.validator_name(self.span)
      }
    }
  }

  pub fn output_proto_type(&self, is_oneof: bool) -> Type {
    match &self.proto_field {
      ProtoField::Map(map) => {
        let keys = map.keys.into_type().output_proto_type(self.span);
        let values = map.values.output_proto_type(self.span);

        parse_quote_spanned! {self.span=> std::collections::HashMap<#keys, #values> }
      }
      ProtoField::Oneof(OneofInfo { path, .. }) => {
        parse_quote_spanned! {self.span=> Option<#path> }
      }
      ProtoField::Repeated(inner) => {
        let inner_type = inner.output_proto_type(self.span);

        parse_quote_spanned! {self.span=> Vec<#inner_type> }
      }
      ProtoField::Optional(inner) => {
        let inner_type = inner.output_proto_type(self.span);

        parse_quote_spanned! {self.span=> Option<#inner_type> }
      }
      ProtoField::Single(inner) => {
        let output_type = inner.output_proto_type(self.span);

        if inner.is_message() && !is_oneof {
          parse_quote_spanned! {self.span=> Option<#output_type> }
        } else {
          parse_quote_spanned! {self.span=> #output_type }
        }
      }
    }
  }
}
