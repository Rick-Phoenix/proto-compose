use std::sync::{Arc, OnceLock};

use ::cel::{Context, ExecutionError, Program, Value};
use cel_rule_builder::{IsComplete, State};
use chrono::Utc;
use proto_types::cel::CelConversionError;
use thiserror::Error;

use super::*;
use crate::validators::field_context::Violations;

pub type CachedProgram = LazyLock<CelProgram>;

pub(crate) fn collect_programs<I>(programs: I) -> Vec<&'static CelProgram>
where
  I: IntoIterator<Item = &'static LazyLock<CelProgram>>,
{
  programs
    .into_iter()
    .map(|lazy_lock| &**lazy_lock)
    .collect()
}

#[doc(hidden)]
pub enum CelTarget<'a> {
  Message,
  Field(&'a FieldContext<'a>),
}

#[derive(Debug)]
pub struct CelProgram {
  pub rule: CelRule,
  pub(crate) program: OnceLock<Program>,
}

impl PartialEq for CelProgram {
  fn eq(&self, other: &Self) -> bool {
    self.rule == other.rule
  }
}

impl CelError {
  // This is for runtime errors. If we get a CEL error we log the actual error while
  // producing a generic error message
  pub fn into_violation(
    self,
    rule: Option<&CelRule>,
    field_context: Option<&FieldContext>,
    parent_elements: &[FieldPathElement],
  ) -> Violation {
    if let Some(rule) = rule {
      log::error!("error with CEL rule with id `{}`: {self}", rule.id);
    } else {
      log::error!("{self}");
    }

    create_violation_core(
      rule.map(|r| r.id.as_ref()),
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
  #[error("failed to initialize context: {0}")]
  ConversionError(#[from] CelConversionError),
  #[error("failed to execute program: {0}")]
  ExecutionError(#[from] ExecutionError),
}

pub fn initialize_context<'a, T, E>(value: T) -> Result<Context<'a>, CelError>
where
  T: TryInto<Value, Error = E>,
  CelConversionError: From<E>,
{
  let mut ctx = Context::default();

  ctx.add_variable_from_value(
    "this",
    value
      .try_into()
      .map_err(|e| CelError::ConversionError(e.into()))?,
  );
  ctx.add_variable_from_value("now", Value::Timestamp(Utc::now().into()));

  Ok(ctx)
}

pub struct ProgramsExecutionCtx<'a, T> {
  pub programs: &'a [&'a CelProgram],
  pub value: T,
  pub violations: &'a mut Vec<Violation>,
  pub field_context: Option<&'a FieldContext<'a>>,
  pub parent_elements: &'a [FieldPathElement],
}

pub fn execute_cel_programs<T, E>(ctx: ProgramsExecutionCtx<T>)
where
  T: TryInto<Value, Error = E>,
  CelConversionError: From<E>,
{
  let ProgramsExecutionCtx {
    programs,
    value,
    violations,
    field_context,
    parent_elements,
  } = ctx;

  let ctx = match initialize_context(value) {
    Ok(ctx) => ctx,
    Err(e) => {
      violations.push(e.into_violation(None, field_context, parent_elements));
      return;
    }
  };

  for program in programs {
    match program.execute(&ctx) {
      Ok(was_successful) => {
        if !was_successful {
          violations.add_cel(&program.rule, field_context, parent_elements);
        }
      }
      Err(e) => {
        violations.push(e.into_violation(Some(&program.rule), field_context, parent_elements))
      }
    };
  }
}

pub fn test_programs<T, E>(programs: &[&CelProgram], value: T) -> Result<(), Vec<CelError>>
where
  T: TryInto<Value, Error = E>,
  CelConversionError: From<E>,
{
  let mut errors: Vec<CelError> = Vec::new();

  let ctx = match initialize_context(value) {
    Ok(ctx) => ctx,
    Err(e) => {
      errors.push(e);
      return Err(errors);
    }
  };

  for program in programs {
    if let Err(e) = program.execute(&ctx) {
      errors.push(e);
    }
  }

  if errors.is_empty() {
    Ok(())
  } else {
    Err(errors)
  }
}

impl CelProgram {
  pub fn new(rule: CelRule) -> Self {
    Self {
      rule,
      program: OnceLock::new(),
    }
  }

  pub fn get_program(&self) -> &Program {
    self.program.get_or_init(|| {
      Program::compile(self.rule.expression.as_ref()).unwrap_or_else(|e| {
        panic!(
          "failed to compile CEL program for rule {}: {e}",
          self.rule.id
        )
      })
    })
  }

  pub fn execute(&self, ctx: &Context) -> Result<bool, CelError> {
    let program = self.get_program();

    let result = program.execute(ctx)?;

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
