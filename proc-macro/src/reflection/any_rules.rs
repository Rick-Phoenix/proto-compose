use super::*;

impl RulesCtx {
  pub fn get_any_validator(&self) -> BuilderTokens {
    let span = self.field_span;
    let mut builder = BuilderTokens::new(span, quote_spanned! {span=> AnyValidator::builder() });

    self.tokenize_ignore(&mut builder);
    self.tokenize_required(&mut builder);
    self.tokenize_cel_rules(&mut builder);

    if let Some(RulesType::Any(rules)) = &self.rules.r#type {
      let in_list = &rules.r#in;
      if !in_list.is_empty() {
        builder.extend(quote_spanned! {span=> .in_([ #(#in_list),* ]) });
      }

      let not_in_list = &rules.not_in;
      if !not_in_list.is_empty() {
        builder.extend(quote_spanned! {span=> .not_in([ #(#not_in_list),* ]) });
      }
    }

    builder
  }
}
