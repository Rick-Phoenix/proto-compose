use crate::*;

#[non_exhaustive]
#[derive(Debug, Error, PartialEq)]
pub enum ConsistencyError {
  #[error("`const` cannot be used with other rules")]
  ConstWithOtherRules,
  #[error(transparent)]
  OverlappingLists(#[from] OverlappingListsError),
  #[error(transparent)]
  CelError(#[from] CelError),
  #[error("{0}")]
  ContradictoryInput(String),
  #[error("The custom messages with these IDs are never used: {0:?}")]
  UnusedCustomMessages(Vec<String>),
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
  #[inline(never)]
  #[cold]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
  pub top_level_errors: Vec<ConsistencyError>,
}

impl Display for MessageTestError {
  #[inline(never)]
  #[cold]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    let Self {
      message_full_name,
      field_errors,
      top_level_errors,
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

    if !top_level_errors.is_empty() {
      let _ = writeln!(f, "  Errors from top level validators:");
      for err in top_level_errors {
        let _ = writeln!(f, "    - {err}");
      }
    }

    Ok(())
  }
}

#[derive(Debug, PartialEq, Eq)]
pub struct OverlappingListsError {
  pub overlapping: Vec<String>,
}

impl core::error::Error for OverlappingListsError {}

impl Display for OverlappingListsError {
  #[inline(never)]
  #[cold]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    writeln!(f, "The following values are both allowed and forbidden:")?;

    for item in &self.overlapping {
      let _ = writeln!(f, "  - {item}");
    }

    Ok(())
  }
}

#[allow(clippy::useless_let_if_seq)]
pub(crate) fn check_comparable_rules<T>(
  lt: Option<T>,
  lte: Option<T>,
  gt: Option<T>,
  gte: Option<T>,
) -> Result<(), ConsistencyError>
where
  T: Display + PartialEq + PartialOrd + Copy,
{
  let mut err: Option<&str> = None;

  if lt.is_some() && lte.is_some() {
    err = Some("Lt and Lte cannot be used together.");
  }

  if gt.is_some() && gte.is_some() {
    err = Some("Gt and Gte cannot be used together.");
  }

  if let Some(lt) = lt {
    if let Some(gt) = gt
      && lt <= gt
    {
      err = Some("Lt cannot be smaller than or equal to Gt");
    }

    if let Some(gte) = gte
      && lt <= gte
    {
      err = Some("Lt cannot be smaller than or equal to Gte");
    }
  }

  if let Some(lte) = lte {
    if let Some(gt) = gt
      && lte <= gt
    {
      err = Some("Lte cannot be smaller than or equal to Gt");
    }

    if let Some(gte) = gte
      && lte < gte
    {
      err = Some("Lte cannot be smaller than Gte");
    }
  }

  if let Some(err) = err {
    Err(ConsistencyError::ContradictoryInput(err.to_string()))
  } else {
    Ok(())
  }
}

pub(crate) fn check_list_rules<T>(
  in_list: Option<&SortedList<T>>,
  not_in_list: Option<&SortedList<T>>,
) -> Result<(), OverlappingListsError>
where
  T: Debug + PartialEq + Eq + core::hash::Hash + Ord + Clone + ListFormatter,
{
  if let Some(in_list) = in_list
    && let Some(not_in_list) = not_in_list
  {
    let mut overlapping: Vec<T> = Vec::with_capacity(in_list.items.len());

    for item in in_list {
      let is_overlapping = not_in_list.items.contains(item);

      if is_overlapping {
        overlapping.push(item.clone());
      }
    }

    if overlapping.is_empty() {
      return Ok(());
    } else {
      return Err(OverlappingListsError {
        overlapping: overlapping
          .into_iter()
          .map(|i| format!("{i:#?}"))
          .collect(),
      });
    }
  }

  Ok(())
}

pub(crate) struct LengthRuleValue {
  pub name: &'static str,
  pub value: Option<usize>,
}

pub(crate) fn check_length_rules(
  exact: Option<&LengthRuleValue>,
  min: &LengthRuleValue,
  max: &LengthRuleValue,
) -> Result<(), ConsistencyError> {
  if let Some(exact) = exact
    && exact.value.is_some()
  {
    if min.value.is_some() {
      return Err(ConsistencyError::ContradictoryInput(format!(
        "{} cannot be used with {}",
        exact.name, min.name
      )));
    }

    if max.value.is_some() {
      return Err(ConsistencyError::ContradictoryInput(format!(
        "{} cannot be used with {}",
        exact.name, max.name
      )));
    }
  }

  if let Some(min_value) = min.value
    && let Some(max_value) = max.value
    && min_value > max_value
  {
    return Err(ConsistencyError::ContradictoryInput(format!(
      "{} cannot be greater than {}",
      min.name, max.name
    )));
  }

  Ok(())
}
