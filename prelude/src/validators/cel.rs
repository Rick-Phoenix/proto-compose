mod cel_trait;
pub use cel_trait::*;

use super::*;

// This will be included even without the cel feature, as it is useful for schema purposes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CelRule {
  /// The id of this specific rule.
  pub id: SharedStr,
  /// The error message to display in case the rule fails validation.
  pub message: SharedStr,
  /// The CEL expression that must be used to perform the validation check.
  pub expression: SharedStr,
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
      name: "cel".into(),
      value: value.into(),
    }
  }
}

impl From<CelRule> for OptionValue {
  fn from(value: CelRule) -> Self {
    Self::Message(
      [
        (SharedStr::Static("id"), Self::String(value.id)),
        (SharedStr::Static("message"), Self::String(value.message)),
        (
          SharedStr::Static("expression"),
          Self::String(value.expression),
        ),
      ]
      .into_iter()
      .collect(),
    )
  }
}

// Without the cel feature, this is just a wrapper for a cel rule

#[cfg(not(feature = "cel"))]
#[derive(Clone, Debug)]
pub struct CelProgram {
  pub rule: CelRule,
}

#[cfg(not(feature = "cel"))]
impl CelProgram {
  pub fn new(rule: CelRule) -> Self {
    Self { rule }
  }
}

#[cfg(not(feature = "cel"))]
#[derive(PartialEq, Eq, Debug, Clone, Error)]
#[error("")]
pub struct CelError;

#[cfg(feature = "cel")]
pub use cel_impls::*;

#[cfg(feature = "cel")]
mod cel_impls {
  use super::*;

  use ::cel::{Context, ExecutionError, Program, Value, objects::ValueType};
  use chrono::Utc;
  use core::convert::Infallible;
  use std::sync::OnceLock;

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
    pub(crate) fn into_violation(
      self,
      field_context: Option<&FieldContext>,
      parent_elements: &[FieldPathElement],
    ) -> Violation {
      // We try to provide more context for this variant
      // since it lacks a program id
      if matches!(self, Self::ConversionError(_)) {
        let mut item_path = String::new();

        for name in parent_elements
          .iter()
          .filter_map(|e| e.field_name.as_ref())
        {
          if !item_path.is_empty() {
            item_path.push('.');
          }
          item_path.push_str(name);
        }

        if let Some(field_name) = field_context.map(|fc| fc.proto_name) {
          if !item_path.is_empty() {
            item_path.push('.');
          }
          item_path.push_str(field_name);
        }

        if item_path.is_empty() {
          item_path.push_str("unknown");
        }

        // A conversion error is due to user input so we don't assign a great priority to it
        log::trace!("Cel execution failure for item at location `{item_path}`: {self}");
      } else {
        // Caused from a malformed CEL expression, so
        // it has higher priority
        log::error!("{self}");
      }

      create_violation_core(
        self.rule_id(),
        field_context,
        parent_elements,
        CEL_VIOLATION,
        "internal server error".to_string(),
      )
    }
  }

  #[derive(Debug, Clone, Error)]
  pub enum CelError {
    #[error("Expected CEL program with id `{rule_id}` to return a boolean result, got `{value:?}`")]
    NonBooleanResult {
      rule_id: SharedStr,
      value: ValueType,
    },
    #[error("Failed to inject value in CEL program: {0}")]
    ConversionError(String),
    #[error("Failed to execute CEL program with id `{rule_id}`: {source}")]
    ExecutionError {
      rule_id: SharedStr,
      source: Box<ExecutionError>,
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

    ctx.add_variable_from_value("this", value.__try_into_cel()?);
    #[cfg(all(feature = "chrono", any(feature = "std", feature = "chrono-wasm")))]
    ctx.add_variable_from_value("now", Value::Timestamp(Utc::now().into()));

    Ok(ctx)
  }

  pub struct ProgramsExecutionCtx<'a, CelT> {
    pub programs: &'a [CelProgram],
    pub value: CelT,
    pub ctx: &'a mut ValidationCtx,
  }

  impl<CelT> ProgramsExecutionCtx<'_, CelT>
  where
    CelT: TryIntoCel,
  {
    pub fn execute_programs(self) -> ValidatorResult {
      let Self {
        programs,
        value,
        ctx,
      } = self;

      let mut is_valid = IsValid::Yes;

      let cel_ctx = match initialize_context(value) {
        Ok(cel_ctx) => cel_ctx,
        Err(e) => {
          let _ = ctx.add_cel_error_violation(e);
          return Err(FailFast);
        }
      };

      for program in programs {
        match program.execute(&cel_ctx) {
          Ok(was_successful) => {
            if !was_successful {
              is_valid &= ctx.add_cel_violation(&program.rule)?;
            }
          }
          Err(e) => is_valid &= ctx.add_cel_error_violation(e)?,
        };
      }

      Ok(is_valid)
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
    pub const fn new(rule: CelRule) -> Self {
      Self {
        rule,
        program: OnceLock::new(),
      }
    }

    // Potentially making this a result too, even with the automated tests
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
          source: Box::new(e),
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
