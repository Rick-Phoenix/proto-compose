use ::proto_types::FieldMask;
use ::proto_types::protovalidate::FieldMaskRules;

use super::*;

pub fn get_field_mask_validator(rules: &FieldMaskRules, ctx: &super::RulesCtx) -> TokenStream2 {
  let mut validator = quote! { ::prelude::FieldMaskValidator::builder() };

  ctx.ignore.tokenize_always_only(&mut validator);
  ctx.tokenize_required(&mut validator);

  if let Some(val) = &rules.r#const {
    let paths = &val.paths;

    validator.extend(quote! { .const_([ #(#paths),* ]) });
  }

  let in_list = &rules.r#in;
  if !in_list.is_empty() {
    validator.extend(quote! { .in_([ #(#in_list),* ]) });
  }

  let not_in_list = &rules.not_in;
  if !not_in_list.is_empty() {
    validator.extend(quote! { .not_in([ #(#not_in_list),* ]) });
  }

  ctx.tokenize_cel_rules(&mut validator);

  quote! { #validator.build() }
}
