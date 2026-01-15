use crate::*;

impl FieldData {
  pub fn consistency_check_tokens(&self) -> Option<TokenStream2> {
    self
      .validator
      .as_ref()
      // Useless to check consistency for default validators
      .filter(|v| !v.is_fallback)
      .map(|validator| {
        let ident_str = &self.ident_str;

        let call = if let ProtoField::Map(_) = &self.proto_field {

          let validator_name = self.validator_name();
          let validator_target_type = self.proto_field.validator_target_type(self.span);

          quote_spanned! {self.span=>
            <#validator_name as ::prelude::Validator<#validator_target_type>>::check_consistency(&#validator)
          }
        } else {
          quote_spanned! {self.span=> ::prelude::Validator::check_consistency(&#validator)}
        };

        quote_spanned! {self.span=>
          if let Err(errs) = #call {
            field_errors.push(::prelude::FieldError {
              field: #ident_str,
              errors: errs
            });
          }
        }
      })
  }

  pub fn descriptor_type_tokens(&self) -> TokenStream2 {
    match &self.proto_field {
      ProtoField::Map(_) => {
        quote_spanned! {self.span=> ::prelude::proto_types::field_descriptor_proto::Type::Message }
      }
      ProtoField::Repeated(inner) | ProtoField::Optional(inner) | ProtoField::Single(inner) => {
        inner.descriptor_type_tokens(self.span)
      }
      ProtoField::Oneof { .. } => {
        quote_spanned! {self.span=> compile_error!("Validator tokens should not be triggered for a oneof field, please report this bug if you see it") }
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

        if map.is_btree_map {
          quote_spanned! {self.span=> btree_map = #map_attr }
        } else {
          quote_spanned! {self.span=> map = #map_attr }
        }
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

        quote_spanned! {self.span=> ::prelude::MapValidator<#keys, #values> }
      }
      ProtoField::Oneof { .. } => quote! {},
      ProtoField::Repeated(proto_type) => {
        let inner = proto_type.validator_target_type(self.span);

        quote_spanned! {self.span=> ::prelude::RepeatedValidator<#inner> }
      }
      ProtoField::Optional(proto_type) | ProtoField::Single(proto_type) => {
        proto_type.validator_name(self.span)
      }
    }
  }

  pub fn output_proto_type(&self, item_kind: ItemKind) -> Type {
    match &self.proto_field {
      ProtoField::Map(map) => {
        let keys = map.keys.into_type().output_proto_type(self.span);
        let values = map.values.output_proto_type(self.span);

        if map.is_btree_map {
          parse_quote_spanned! {self.span=> ::prelude::BTreeMap<#keys, #values> }
        } else {
          parse_quote_spanned! {self.span=> ::std::collections::HashMap<#keys, #values> }
        }
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

        if inner.is_message() && item_kind.is_message() {
          parse_quote_spanned! {self.span=> Option<#output_type> }
        } else {
          parse_quote_spanned! {self.span=> #output_type }
        }
      }
    }
  }
}
