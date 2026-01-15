use super::*;

impl RulesCtx {
  pub fn get_map_validator(self, map_data: &ProtoMap) -> BuilderTokens {
    let span = self.field_span;
    let ProtoMap { keys, values, .. } = map_data;

    let keys_validator_type = keys.into_type().validator_target_type(span);
    let values_validator_type = values.validator_target_type(span);
    let mut builder = BuilderTokens::new(
      span,
      quote_spanned! {span=> MapValidator::<#keys_validator_type, #values_validator_type>::builder() },
    );

    self.tokenize_ignore(&mut builder);
    self.tokenize_cel_rules(&mut builder);

    if let Some(RulesType::Map(rules)) = self.rules.r#type {
      if let Some(val) = rules.min_pairs {
        #[allow(clippy::cast_possible_truncation)]
        let val = val as usize;

        builder.extend(quote_spanned! {span=> .min_pairs(#val) });
      }

      if let Some(val) = rules.max_pairs {
        #[allow(clippy::cast_possible_truncation)]
        let val = val as usize;

        builder.extend(quote_spanned! {span=> .max_pairs(#val) });
      }

      if let Some(keys_rules) = rules
        .keys
        .and_then(|r| Self::from_non_empty_rules(*r, self.field_span))
      {
        let keys_validator = keys_rules
          .get_field_validator(&((*keys).into()))
          .into_builder();

        builder.extend(quote_spanned! {span=> .keys(|_| #keys_validator) });
      }

      if let Some(values_rules) = rules
        .values
        .and_then(|r| Self::from_non_empty_rules(*r, self.field_span))
      {
        let values_validator = values_rules
          .get_field_validator(values)
          .into_builder();

        builder.extend(quote_spanned! {span=> .values(|_| #values_validator) });
      }
    }

    builder
  }
}
