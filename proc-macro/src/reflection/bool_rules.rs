use super::*;

impl RulesCtx {
  pub fn get_bool_validator(&self) -> BuilderTokens {
    let span = self.field_span;
    let mut builder = BuilderTokens::new(span, quote_spanned! {span=> BoolValidator::builder() });

    self.tokenize_ignore(&mut builder);
    self.tokenize_required(&mut builder);

    if let Some(RulesType::Bool(rules)) = &self.rules.r#type
      && let Some(val) = rules.r#const
    {
      builder.extend(quote_spanned! {span=> .const_(#val) });
    }

    builder
  }
}
