mod cel_trait;
pub use cel_trait::*;

use super::*;

// This will be included even without the cel feature, as it is useful for schema purposes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CelRule {
  /// The id of this specific rule.
  pub id: Arc<str>,
  /// The error message to display in case the rule fails validation.
  pub message: Arc<str>,
  /// The CEL expression that must be used to perform the validation check.
  pub expression: Arc<str>,
}

impl From<CelRule> for CelProgram {
  #[inline]
  fn from(value: CelRule) -> Self {
    Self::new(value)
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
    Self::Message(
      [
        (ID.clone(), Self::String(value.id)),
        (MESSAGE.clone(), Self::String(value.message)),
        (EXPRESSION.clone(), Self::String(value.expression)),
      ]
      .into_iter()
      .collect(),
    )
  }
}

// Without the cel feature, this is just a wrapper for a cel rule

#[cfg(not(feature = "cel"))]
#[derive(Debug)]
pub struct CelProgram {
  pub rule: CelRule,
}

#[cfg(not(feature = "cel"))]
impl CelProgram {
  pub fn new(rule: CelRule) -> Self {
    Self { rule }
  }
}

#[cfg(feature = "cel")]
pub use cel_impls::*;

#[cfg(feature = "cel")]
mod cel_impls {
  use super::*;

  use ::cel::{Context, ExecutionError, Program, Value, objects::ValueType};
  use chrono::Utc;
  use std::{convert::Infallible, sync::OnceLock};

  #[derive(Debug)]
  pub struct CelProgram {
    pub rule: CelRule,
    program: OnceLock<Program>,
  }

  impl Clone for CelProgram {
    #[inline]
    fn clone(&self) -> Self {
      Self {
        rule: self.rule.clone(),
        program: OnceLock::new(),
      }
    }
  }

  impl PartialEq for CelProgram {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
      self.rule == other.rule
    }
  }

  impl CelError {
    #[must_use]
    #[inline]
    pub fn rule_id(&self) -> Option<&str> {
      match self {
        Self::ConversionError(_) => None,
        Self::NonBooleanResult { rule_id, .. } | Self::ExecutionError { rule_id, .. } => {
          Some(rule_id.as_ref())
        }
      }
    }

    // This is for runtime errors. If we get a CEL error we log the actual error while
    // producing a generic error message
    #[must_use]
    #[inline]
    pub fn into_violation(
      self,
      field_context: Option<&FieldContext>,
      parent_elements: &[FieldPathElement],
    ) -> Violation {
      log::error!("{self}");

      create_violation_core(
        self.rule_id(),
        field_context,
        parent_elements,
        &CEL_VIOLATION,
        "internal server error",
      )
    }
  }

  #[derive(Debug, Clone, Error)]
  pub enum CelError {
    #[error("Expected CEL program with id `{rule_id}` to return a boolean result, got `{value:?}`")]
    NonBooleanResult { rule_id: Arc<str>, value: ValueType },
    // SHould use FieldPath here to at least get the context of the value
    #[error("Failed to inject value in CEL program: {0}")]
    ConversionError(String),
    #[error("Failed to execute CEL program with id `{rule_id}`: {source}")]
    ExecutionError {
      rule_id: Arc<str>,
      source: ExecutionError,
    },
  }

  const fn partial_eq_value_type(input: ValueType, other: ValueType) -> bool {
    match input {
      ValueType::List => matches!(other, ValueType::List),
      ValueType::Map => matches!(other, ValueType::Map),
      ValueType::Function => matches!(other, ValueType::Function),
      ValueType::Int => matches!(other, ValueType::Int),
      ValueType::UInt => matches!(other, ValueType::UInt),
      ValueType::Float => matches!(other, ValueType::Float),
      ValueType::String => matches!(other, ValueType::String),
      ValueType::Bytes => matches!(other, ValueType::Bytes),
      ValueType::Bool => matches!(other, ValueType::Bool),
      ValueType::Duration => matches!(other, ValueType::Duration),
      ValueType::Timestamp => matches!(other, ValueType::Timestamp),
      ValueType::Opaque => matches!(other, ValueType::Opaque),
      ValueType::Null => matches!(other, ValueType::Null),
    }
  }

  impl PartialEq for CelError {
    fn eq(&self, other: &Self) -> bool {
      match self {
        Self::NonBooleanResult { rule_id, value } => {
          if let Self::NonBooleanResult {
            rule_id: other_rule_id,
            value: other_value,
          } = other
          {
            rule_id == other_rule_id && partial_eq_value_type(*value, *other_value)
          } else {
            false
          }
        }
        Self::ConversionError(err) => {
          if let Self::ConversionError(other_err) = other {
            err == other_err
          } else {
            false
          }
        }
        Self::ExecutionError { rule_id, source } => {
          if let Self::ExecutionError {
            rule_id: other_rule_id,
            source: other_source,
          } = other
          {
            rule_id == other_rule_id && source == other_source
          } else {
            false
          }
        }
      }
    }
  }

  impl From<Infallible> for CelError {
    #[inline]
    fn from(value: Infallible) -> Self {
      match value {}
    }
  }

  fn initialize_context<'a, T>(value: T) -> Result<Context<'a>, CelError>
  where
    T: TryIntoCel,
  {
    let mut ctx = Context::default();

    ctx.add_variable_from_value("this", value.try_into_cel()?);
    ctx.add_variable_from_value("now", Value::Timestamp(Utc::now().into()));

    Ok(ctx)
  }

  pub struct ProgramsExecutionCtx<'a, CelT> {
    pub programs: &'a [CelProgram],
    pub value: CelT,
    pub violations: &'a mut ViolationsAcc,
    pub field_context: Option<&'a FieldContext<'a>>,
    pub parent_elements: &'a [FieldPathElement],
  }

  impl<CelT> ProgramsExecutionCtx<'_, CelT>
  where
    CelT: TryIntoCel,
  {
    pub fn execute_programs(self) {
      let Self {
        programs,
        value,
        violations,
        field_context,
        parent_elements,
      } = self;

      let ctx = match initialize_context(value) {
        Ok(ctx) => ctx,
        Err(e) => {
          violations.push(e.into_violation(field_context, parent_elements));
          return;
        }
      };

      for program in programs {
        match program.execute(&ctx) {
          Ok(was_successful) => {
            if !was_successful {
              violations.add_cel_violation(&program.rule, field_context, parent_elements);
            }
          }
          Err(e) => violations.push(e.into_violation(field_context, parent_elements)),
        };
      }
    }
  }

  pub fn test_programs<T>(programs: &[CelProgram], value: T) -> Result<(), Vec<CelError>>
  where
    T: TryIntoCel,
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
    #[must_use]
    #[inline]
    pub const fn new(rule: CelRule) -> Self {
      Self {
        rule,
        program: OnceLock::new(),
      }
    }

    // Potentially making this a result too, even with the automated tests
    #[inline]
    pub fn get_program(&self) -> &Program {
      self.program.get_or_init(|| {
        Program::compile(&self.rule.expression).unwrap_or_else(|e| {
          panic!(
            "Failed to compile CEL program with id `{}`: {e}",
            self.rule.id
          )
        })
      })
    }

    pub fn execute(&self, ctx: &Context) -> Result<bool, CelError> {
      let program = self.get_program();

      let result = program
        .execute(ctx)
        .map_err(|e| CelError::ExecutionError {
          rule_id: self.rule.id.clone(),
          source: e,
        })?;

      if let Value::Bool(result) = result {
        Ok(result)
      } else {
        Err(CelError::NonBooleanResult {
          rule_id: self.rule.id.clone(),
          value: result.type_of(),
        })
      }
    }
  }
}
