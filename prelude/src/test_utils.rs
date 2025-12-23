use crate::*;

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
