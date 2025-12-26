use cel_rule_builder::{IsComplete, State};

use super::*;
use crate::validators::field_context::ViolationsExt;

// This will be included even without the cel feature, as it is useful for schema purposes
#[derive(Debug, Clone, Builder, PartialEq, Eq)]
pub struct CelRule {
  /// The id of this specific rule.
  pub id: &'static str,
  /// The error message to display in case the rule fails validation.
  pub message: &'static str,
  /// The CEL expression that must be used to perform the validation check.
  pub expression: &'static str,
}

// Not being detected by the LSP that this is used
#[allow(unused)]
#[cfg(feature = "cel")]
pub trait IntoCel: Into<::cel::Value> {}
#[cfg(feature = "cel")]
impl<T: Into<::cel::Value>> IntoCel for T {}

#[cfg(not(feature = "cel"))]
pub trait IntoCel {}
#[cfg(not(feature = "cel"))]
impl<T> IntoCel for T {}

impl From<CelRule> for CelProgram {
  fn from(value: CelRule) -> Self {
    Self::new(value)
  }
}

impl<S> From<CelRuleBuilder<S>> for OptionValue
where
  S: State + IsComplete,
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
    Self::Message(
      vec![
        (ID.clone(), Self::String(value.id.into())),
        (MESSAGE.clone(), Self::String(value.message.into())),
        (EXPRESSION.clone(), Self::String(value.expression.into())),
      ]
      .into(),
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
  use proto_types::cel::CelConversionError;
  use std::sync::OnceLock;

  #[derive(Debug)]
  pub struct CelProgram {
    pub rule: CelRule,
    program: OnceLock<Program>,
  }

  pub type CachedProgram = LazyLock<CelProgram>;

  impl PartialEq for CelProgram {
    fn eq(&self, other: &Self) -> bool {
      self.rule == other.rule
    }
  }

  impl CelError {
    #[must_use]
    pub const fn rule_id(&self) -> Option<&'static str> {
      match self {
        Self::ConversionError(_) => None,
        Self::NonBooleanResult { rule_id, .. } | Self::ExecutionError { rule_id, .. } => {
          Some(rule_id)
        }
      }
    }

    // This is for runtime errors. If we get a CEL error we log the actual error while
    // producing a generic error message
    #[must_use]
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
    NonBooleanResult {
      rule_id: &'static str,
      value: ValueType,
    },
    // SHould use FieldPath here to at least get the context of the value
    #[error("Failed to inject value in CEL program: {0}")]
    ConversionError(#[from] CelConversionError),
    #[error("Failed to execute CEL program with id `{rule_id}`: {source}")]
    ExecutionError {
      rule_id: &'static str,
      source: ExecutionError,
    },
  }

  fn initialize_context<'a, T, E>(value: T) -> Result<Context<'a>, CelError>
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

  impl<T, E> ProgramsExecutionCtx<'_, T>
  where
    T: TryInto<Value, Error = E>,
    CelConversionError: From<E>,
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
              violations.add_cel(&program.rule, field_context, parent_elements);
            }
          }
          Err(e) => violations.push(e.into_violation(field_context, parent_elements)),
        };
      }
    }
  }

  #[cfg(feature = "testing")]
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
    #[must_use]
    pub const fn new(rule: CelRule) -> Self {
      Self {
        rule,
        program: OnceLock::new(),
      }
    }

    // Potentially making this a result too, even with the automated tests
    pub fn get_program(&self) -> &Program {
      self.program.get_or_init(|| {
        Program::compile(self.rule.expression).unwrap_or_else(|e| {
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
          rule_id: self.rule.id,
          source: e,
        })?;

      if let Value::Bool(result) = result {
        Ok(result)
      } else {
        Err(CelError::NonBooleanResult {
          rule_id: self.rule.id,
          value: result.type_of(),
        })
      }
    }
  }
}
