use std::sync::Arc;

use cel_rule_builder::{IsComplete, State};

use super::*;

/// The structure of a custom Cel rule used to define validation logic with protovalidate.
/// The id should be a unique identifier for the given rule, which will appear in error messages.
/// The message is the error message that will show up in case of validation failure.
/// The expression is a [Cel](https://cel.dev/) expression that will be executed on validation by protovalidate-compatible libraries such as [protocheck](https://github.com/Rick-Phoenix/protocheck) (for rust) or [protovalidate-es](https://github.com/bufbuild/protovalidate-es) (for javascript).
/// <br/>
/// You can use the [`cel_rule`](crate::cel_rule) macro to build them with a shorter syntax.
#[derive(Debug, Clone, Builder, PartialEq)]
#[builder(on(Arc<str>, into))]
pub struct CelRule {
  /// The id of this specific rule.
  pub id: Arc<str>,
  /// The error message to display in case the rule fails validation.
  pub message: Arc<str>,
  /// The CEL expression that must be used to perform the validation check.
  pub expression: Arc<str>,
}

impl<S: State> From<CelRuleBuilder<S>> for OptionValue
where
  S: IsComplete,
{
  fn from(value: CelRuleBuilder<S>) -> Self {
    value.build().into()
  }
}

impl From<CelRule> for ProtoOption {
  fn from(value: CelRule) -> Self {
    Self {
      name: CEL.clone(),
      value: value.into(),
    }
  }
}

impl From<CelRule> for OptionValue {
  fn from(value: CelRule) -> Self {
    OptionValue::Message(
      vec![
        (ID.clone(), OptionValue::String(value.id)),
        (MESSAGE.clone(), OptionValue::String(value.message)),
        (EXPRESSION.clone(), OptionValue::String(value.expression)),
      ]
      .into(),
    )
  }
}
