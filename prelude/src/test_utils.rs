use crate::*;

#[derive(Debug, Error)]
pub enum ConsistencyError {
  #[error("`const` cannot be used with other rules")]
  ConstWithOtherRules,
  #[error(transparent)]
  OverlappingLists(#[from] OverlappingListsError),
  #[error(transparent)]
  CelError(#[from] CelError),
  #[error("{0}")]
  ContradictoryInput(String),
  #[error("{0}")]
  WrongOneofTags(String),
}

#[derive(Debug)]
pub struct FieldError {
  pub field: &'static str,
  pub errors: Vec<ConsistencyError>,
}

#[derive(Debug)]
pub struct OneofErrors {
  pub oneof_name: &'static str,
  pub field_errors: Vec<FieldError>,
}

impl core::error::Error for OneofErrors {}

impl Display for OneofErrors {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let _ = writeln!(
      f,
      "❌ Validator consistency check for oneof `{}` has failed:",
      self.oneof_name.bright_yellow()
    );

    for field_error in &self.field_errors {
      let _ = writeln!(
        f,
        "  {}{}{}:",
        self.oneof_name.bright_cyan(),
        "::".bright_cyan(),
        field_error.field.bright_cyan()
      );

      for err in &field_error.errors {
        let _ = writeln!(f, "        - {err}");
      }
    }

    Ok(())
  }
}

pub struct MessageTestError {
  pub message_full_name: &'static str,
  pub field_errors: Vec<FieldError>,
  pub cel_errors: Vec<CelError>,
}

impl Display for MessageTestError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let Self {
      message_full_name,
      field_errors,
      cel_errors,
    } = self;

    let _ = writeln!(
      f,
      "❌ Validator consistency check for message `{}` has failed:",
      message_full_name.bright_yellow()
    );

    if !field_errors.is_empty() {
      let _ = writeln!(f, "  Fields errors:");

      for field_error in field_errors {
        let _ = writeln!(f, "    {}:", field_error.field.bright_yellow());

        for err in &field_error.errors {
          let _ = writeln!(f, "      - {err}");
        }
      }
    }

    if !cel_errors.is_empty() {
      let _ = writeln!(f, "  CEL rules errors:");
      for err in cel_errors {
        let _ = writeln!(f, "    - {err}");
      }
    }

    Ok(())
  }
}
