use ::proto_types::Timestamp;
use ::proto_types::protovalidate::TimestampRules;
use ::proto_types::protovalidate::timestamp_rules::{GreaterThan, LessThan};

use super::*;

fn tokenize_timestamp(timestamp: Timestamp) -> TokenStream2 {
  let Timestamp { seconds, nanos } = timestamp;

  quote! { ::prelude::proto_types::Timestamp { seconds: #seconds, nanos: #nanos } }
}

pub fn get_timestamp_validator(rules: &TimestampRules, ctx: &super::RulesCtx) -> TokenStream2 {
  let mut validator = quote! { ::prelude::TimestampValidator::builder() };

  ctx.ignore.tokenize_always_only(&mut validator);
  ctx.tokenize_required(&mut validator);

  if let Some(val) = rules.r#const {
    let val = tokenize_timestamp(val);

    validator.extend(quote! { .const_(#val) });
  }

  if let Some(less_than) = &rules.less_than {
    match less_than {
      LessThan::Lt(val) => {
        let val = tokenize_timestamp(*val);
        validator.extend(quote! { .lt(#val) });
      }
      LessThan::Lte(val) => {
        let val = tokenize_timestamp(*val);
        validator.extend(quote! { .lte(#val) });
      }
      LessThan::LtNow(true) => {
        validator.extend(quote! { .lt_now() });
      }
      _ => {}
    };
  }

  if let Some(greater_than) = &rules.greater_than {
    match greater_than {
      GreaterThan::Gt(val) => {
        let val = tokenize_timestamp(*val);
        validator.extend(quote! { .gt(#val) });
      }
      GreaterThan::Gte(val) => {
        let val = tokenize_timestamp(*val);
        validator.extend(quote! { .gte(#val) });
      }
      GreaterThan::GtNow(true) => {
        validator.extend(quote! { .gt_now() });
      }
      _ => {}
    };
  }

  if let Some(val) = &rules.within {
    let val = tokenize_duration(*val);

    validator.extend(quote! { .within(#val) });
  }

  ctx.tokenize_cel_rules(&mut validator);

  quote! { #validator.build() }
}
