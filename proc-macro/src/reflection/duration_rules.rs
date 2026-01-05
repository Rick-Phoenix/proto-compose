use ::proto_types::Duration;
use ::proto_types::protovalidate::DurationRules;
use ::proto_types::protovalidate::duration_rules::{GreaterThan, LessThan};

use super::*;

pub(super) fn tokenize_duration(duration: Duration) -> TokenStream2 {
  let Duration { seconds, nanos } = duration;

  quote! { ::prelude::proto_types::Duration { seconds: #seconds, nanos: #nanos } }
}

pub fn get_duration_validator(rules: &DurationRules, ctx: &super::RulesCtx) -> TokenStream2 {
  let mut validator = quote! { ::prelude::DurationValidator::builder() };

  ctx.ignore.tokenize_always_only(&mut validator);
  ctx.tokenize_required(&mut validator);

  if let Some(val) = rules.r#const {
    let val = tokenize_duration(val);

    validator.extend(quote! { .const_(#val) });
  }

  if let Some(less_than) = &rules.less_than {
    match less_than {
      LessThan::Lt(val) => {
        let val = tokenize_duration(*val);
        validator.extend(quote! { .lt(#val) });
      }
      LessThan::Lte(val) => {
        let val = tokenize_duration(*val);
        validator.extend(quote! { .lte(#val) });
      }
    };
  }

  if let Some(greater_than) = &rules.greater_than {
    match greater_than {
      GreaterThan::Gt(val) => {
        let val = tokenize_duration(*val);
        validator.extend(quote! { .gt(#val) });
      }
      GreaterThan::Gte(val) => {
        let val = tokenize_duration(*val);
        validator.extend(quote! { .gte(#val) });
      }
    };
  }

  let in_list = &rules.r#in;
  if !in_list.is_empty() {
    let in_list = in_list.iter().map(|d| tokenize_duration(*d));
    validator.extend(quote! { .in_([ #(#in_list),* ]) });
  }

  let not_in_list = &rules.not_in;
  if !not_in_list.is_empty() {
    let not_in_list = not_in_list.iter().map(|d| tokenize_duration(*d));
    validator.extend(quote! { .not_in([ #(#not_in_list),* ]) });
  }

  ctx.tokenize_cel_rules(&mut validator);

  quote! { #validator.build() }
}
