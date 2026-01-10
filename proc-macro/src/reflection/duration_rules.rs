use ::proto_types::Duration;
use ::proto_types::protovalidate::duration_rules::{GreaterThan, LessThan};

use super::*;

pub(super) fn tokenize_duration(span: Span, duration: Duration) -> TokenStream2 {
  let Duration { seconds, nanos } = duration;

  quote_spanned! {span=> ::prelude::proto_types::Duration { seconds: #seconds, nanos: #nanos } }
}

impl RulesCtx<'_> {
  pub fn get_duration_validator(&self) -> BuilderTokens {
    let span = self.field_span;
    let mut builder =
      BuilderTokens::new(span, quote_spanned! {span=> DurationValidator::builder() });

    self.tokenize_ignore(&mut builder);
    self.tokenize_required(&mut builder);
    self.tokenize_cel_rules(&mut builder);

    if let Some(RulesType::Duration(rules)) = &self.rules.r#type {
      if let Some(val) = rules.r#const {
        let val = tokenize_duration(span, val);

        builder.extend(quote_spanned! {span=> .const_(#val) });
      }

      if let Some(less_than) = rules.less_than {
        match less_than {
          LessThan::Lt(val) => {
            let val = tokenize_duration(span, val);
            builder.extend(quote_spanned! {span=> .lt(#val) });
          }
          LessThan::Lte(val) => {
            let val = tokenize_duration(span, val);
            builder.extend(quote_spanned! {span=> .lte(#val) });
          }
        };
      }

      if let Some(greater_than) = rules.greater_than {
        match greater_than {
          GreaterThan::Gt(val) => {
            let val = tokenize_duration(span, val);
            builder.extend(quote_spanned! {span=> .gt(#val) });
          }
          GreaterThan::Gte(val) => {
            let val = tokenize_duration(span, val);
            builder.extend(quote_spanned! {span=> .gte(#val) });
          }
        };
      }

      let in_list = &rules.r#in;
      if !in_list.is_empty() {
        let in_list = in_list
          .iter()
          .map(|d| tokenize_duration(span, *d));
        builder.extend(quote_spanned! {span=> .in_([ #(#in_list),* ]) });
      }

      let not_in_list = &rules.not_in;
      if !not_in_list.is_empty() {
        let not_in_list = not_in_list
          .iter()
          .map(|d| tokenize_duration(span, *d));
        builder.extend(quote_spanned! {span=> .not_in([ #(#not_in_list),* ]) });
      }
    }

    builder
  }
}
