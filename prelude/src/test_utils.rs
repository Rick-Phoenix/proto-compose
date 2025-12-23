use crate::*;

#[must_use]
pub fn format_oneof_errors(
  oneof_name: &'static str,
  errors: Vec<(&'static str, Vec<String>)>,
) -> Vec<String> {
  let mut errors_vec = Vec::new();

  for (variant_name, errs) in errors {
    let mut error = String::new();

    let _ = writeln!(
      error,
      "{}{}{}:",
      oneof_name.bright_cyan(),
      "::".bright_cyan(),
      variant_name.bright_cyan()
    );

    for err in errs {
      let _ = writeln!(error, "        - {err}");
    }

    errors_vec.push(error);
  }

  errors_vec
}

pub struct MessageTestError {
  pub message_full_name: &'static str,
  pub field_errors: Vec<(&'static str, Vec<String>)>,
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

      for (field_name, errs) in field_errors {
        let _ = writeln!(f, "    {}:", field_name.bright_yellow());

        for err in errs {
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

#[must_use]
pub fn format_message_errors(errors: MessageTestError) -> String {
  let MessageTestError {
    message_full_name: message_name,
    field_errors,
    cel_errors,
  } = errors;

  let mut error = String::new();

  writeln!(
    error,
    "❌ Validator consistency check for message `{}` has failed:",
    message_name.bright_yellow()
  )
  .unwrap();

  if !field_errors.is_empty() {
    let _ = writeln!(error, "  Fields errors:");

    for (field_name, errs) in field_errors {
      let _ = writeln!(error, "    {}:", field_name.bright_yellow());

      for err in errs {
        let _ = writeln!(error, "      - {err}");
      }
    }
  }

  if !cel_errors.is_empty() {
    let _ = writeln!(error, "  CEL rules errors:");
    for err in cel_errors {
      let _ = writeln!(error, "    - {err}");
    }
  }

  error
}

#[must_use]
pub fn cel_programs_error(message_name: &str, errors: Vec<CelError>) -> String {
  let mut error = String::new();

  writeln!(
    error,
    "❌ Testing CEL programs for message `{}` has failed:",
    message_name.bright_yellow()
  )
  .unwrap();

  for err in errors {
    writeln!(error, "  - {err}").unwrap();
  }

  error
}
