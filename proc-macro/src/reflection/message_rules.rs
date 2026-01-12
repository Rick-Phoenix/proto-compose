use super::*;

impl RulesCtx {
  pub fn get_message_field_validator(&self, msg_path: &Path) -> BuilderTokens {
    let span = self.field_span;
    let mut builder = BuilderTokens::new(
      span,
      quote_spanned! {span=> MessageValidator::<#msg_path>::builder() },
    );

    self.tokenize_ignore(&mut builder);
    self.tokenize_cel_rules(&mut builder);
    self.tokenize_required(&mut builder);

    builder
  }
}
