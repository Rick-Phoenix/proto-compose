use super::*;

impl RulesCtx {
  pub fn get_repeated_validator(self, inner: &ProtoType) -> BuilderTokens {
    let span = self.field_span;

    let inner_validator_type = inner.validator_target_type(span);
    let mut builder = BuilderTokens::new(
      span,
      quote_spanned! {span=> RepeatedValidator::<#inner_validator_type>::builder() },
    );

    self.tokenize_ignore(&mut builder);
    self.tokenize_cel_rules(&mut builder);

    if let Some(RulesType::Repeated(rules)) = self.rules.r#type {
      if let Some(val) = rules.min_items {
        #[allow(clippy::cast_possible_truncation)]
        let val = val as usize;

        builder.extend(quote_spanned! {span=> .min_items(#val) });
      }

      if let Some(val) = rules.max_items {
        #[allow(clippy::cast_possible_truncation)]
        let val = val as usize;

        builder.extend(quote_spanned! {span=> .max_items(#val) });
      }

      if rules.unique() {
        builder.extend(quote_spanned! {span=> .unique() });
      }

      if let Some(items_rules) = rules
        .items
        .and_then(|r| Self::from_non_empty_rules(*r, self.field_span))
      {
        let items_validator = items_rules
          .get_field_validator(inner)
          .into_builder();

        builder.extend(quote_spanned! {span=> .items(|_| #items_validator) });
      }
    }

    builder
  }
}
