use std::sync::{Arc, OnceLock};

use ::cel::{Context, ExecutionError, Program, Value};
use cel_rule_builder::{IsComplete, State};
use chrono::Utc;
use proto_types::cel::CelConversionError;
use thiserror::Error;

use super::*;

#[doc(hidden)]
pub enum CelTarget<'a> {
  Message,
  Field(&'a FieldContext<'a>),
}

pub struct CelProgram {
  pub rule: CelRule,
  pub(crate) program: OnceLock<Program>,
}

impl CelError {
  pub fn into_violation(
    self,
    rule: &CelRule,
    field_context: Option<&FieldContext>,
    parent_elements: &[FieldPathElement],
  ) -> Violation {
    log::error!("error with CEL rule with id `{}`: {self}", rule.id);

    create_violation_core(
      Some(rule.id.as_ref()),
      field_context,
      parent_elements,
      &CEL_VIOLATION,
      "internal server error",
    )
  }
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum CelError {
  #[error("expected a boolean result, got {0:?}")]
  NonBooleanResult(Value),
  #[error(transparent)]
  ConversionError(#[from] CelConversionError),
  #[error(transparent)]
  ExecutionError(#[from] ExecutionError),
}

impl CelProgram {
  pub fn new(rule: CelRule) -> Self {
    Self {
      rule,
      program: OnceLock::new(),
    }
  }

  pub fn execute(
    &self,
    value: impl TryInto<Value, Error = impl Into<CelConversionError>>,
  ) -> Result<bool, CelError> {
    let program = self.program.get_or_init(|| {
      Program::compile(self.rule.expression.as_ref()).unwrap_or_else(|e| {
        panic!(
          "failed to compile CEL program for rule {}: {e}",
          self.rule.id
        )
      })
    });

    let mut ctx = Context::default();

    ctx.add_variable_from_value("this", value.try_into().map_err(|e| e.into())?);
    ctx.add_variable_from_value("now", Value::Timestamp(Utc::now().into()));

    let result = program.execute(&ctx)?;

    if let Value::Bool(result) = result {
      Ok(result)
    } else {
      Err(CelError::NonBooleanResult(result))
    }
  }
}

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
