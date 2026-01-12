use ::proto_types::Timestamp;
use ::proto_types::protovalidate::timestamp_rules::{GreaterThan, LessThan};

use super::duration_rules::tokenize_duration;
use super::*;

fn tokenize_timestamp(span: Span, timestamp: Timestamp) -> TokenStream2 {
  let Timestamp { seconds, nanos } = timestamp;

  quote_spanned! {span=> ::prelude::proto_types::Timestamp { seconds: #seconds, nanos: #nanos } }
}

impl RulesCtx {
  pub fn get_timestamp_validator(&self) -> BuilderTokens {
    let span = self.field_span;

    let mut builder =
      BuilderTokens::new(span, quote_spanned! {span=> TimestampValidator::builder() });

    self.tokenize_ignore(&mut builder);
    self.tokenize_required(&mut builder);
    self.tokenize_cel_rules(&mut builder);

    if let Some(RulesType::Timestamp(rules)) = &self.rules.r#type {
      if let Some(val) = rules.r#const {
        let val = tokenize_timestamp(span, val);

        builder.extend(quote_spanned! {span=> .const_(#val) });
      }

      if let Some(less_than) = rules.less_than {
        match less_than {
          LessThan::Lt(val) => {
            let val = tokenize_timestamp(span, val);
            builder.extend(quote_spanned! {span=> .lt(#val) });
          }
          LessThan::Lte(val) => {
            let val = tokenize_timestamp(span, val);
            builder.extend(quote_spanned! {span=> .lte(#val) });
          }
          LessThan::LtNow(true) => {
            builder.extend(quote_spanned! {span=> .lt_now() });
          }
          _ => {}
        };
      }

      if let Some(greater_than) = rules.greater_than {
        match greater_than {
          GreaterThan::Gt(val) => {
            let val = tokenize_timestamp(span, val);
            builder.extend(quote_spanned! {span=> .gt(#val) });
          }
          GreaterThan::Gte(val) => {
            let val = tokenize_timestamp(span, val);
            builder.extend(quote_spanned! {span=> .gte(#val) });
          }
          GreaterThan::GtNow(true) => {
            builder.extend(quote_spanned! {span=> .gt_now() });
          }
          _ => {}
        };
      }

      if let Some(val) = rules.within {
        let val = tokenize_duration(span, val);

        builder.extend(quote_spanned! {span=> .within(#val) });
      }
    }

    builder
  }
}
