use super::{field_path_element::Subscript, *};

pub trait Violations {
  fn add(
    &mut self,
    field_context: &FieldContext,
    parent_elements: &[FieldPathElement],
    violation_data: &ViolationData,
    error_message: &str,
  );

  fn add_with_custom_id(
    &mut self,
    rule_id: &str,
    field_context: &FieldContext,
    parent_elements: &[FieldPathElement],
    violation_data: &ViolationData,
    error_message: &str,
  );
}

impl Violations for Vec<Violation> {
  fn add(
    &mut self,
    field_context: &FieldContext,
    parent_elements: &[FieldPathElement],
    violation_data: &ViolationData,
    error_message: &str,
  ) {
    let violation = new_violation(
      field_context,
      parent_elements,
      violation_data,
      error_message,
    );
    self.push(violation);
  }

  fn add_with_custom_id(
    &mut self,
    rule_id: &str,
    field_context: &FieldContext,
    parent_elements: &[FieldPathElement],
    violation_data: &ViolationData,
    error_message: &str,
  ) {
    let violation = new_violation_with_custom_id(
      rule_id,
      field_context,
      parent_elements,
      violation_data,
      error_message,
    );
    self.push(violation);
  }
}

pub trait ValidationResult {
  fn push_violations(self, violations: &mut Vec<Violation>);
}

impl ValidationResult for Result<(), Vec<Violation>> {
  fn push_violations(self, violations: &mut Vec<Violation>) {
    if let Err(new_violations) = self {
      violations.extend(new_violations);
    }
  }
}

impl IntoSubscript for Subscript {
  fn into_subscript(self) -> Subscript {
    self
  }
}

/// The context for the field being validated.
#[derive(Clone, Debug)]
pub struct FieldContext<'a> {
  pub name: &'a str,
  pub tag: i32,
  pub key_type: Option<Type>,
  pub value_type: Option<Type>,
  pub kind: FieldKind,
  pub field_type: Type,
  pub subscript: Option<Subscript>,
}

#[derive(Clone, Default, Debug, Copy, PartialEq, Eq)]
pub enum FieldKind {
  MapKey,
  MapValue,
  RepeatedItem,
  #[default]
  Other,
}

impl FieldKind {
  /// Returns `true` if the field kind is [`MapKey`].
  ///
  /// [`MapKey`]: FieldKind::MapKey
  #[must_use]
  pub fn is_map_key(&self) -> bool {
    matches!(self, Self::MapKey)
  }
}

pub struct ValidationContext {
  pub parent_elements: Vec<FieldPathElement>,
  pub violations: Vec<Violation>,
}

fn create_violation_core(
  custom_rule_id: Option<&str>,
  field_context: &FieldContext,
  parent_elements: &[FieldPathElement],
  violation_data: &ViolationData,
  error_message: &str,
) -> Violation {
  let mut field_elements = parent_elements.to_vec();

  let current_elem = FieldPathElement {
    field_type: Some(field_context.field_type as i32),
    field_name: Some(field_context.name.to_string()),
    key_type: field_context.key_type.map(|t| t as i32),
    value_type: field_context.value_type.map(|t| t as i32),
    field_number: Some(field_context.tag),
    subscript: field_context.subscript.clone(),
  };

  field_elements.push(current_elem);

  let mut rule_elements: Vec<FieldPathElement> = Vec::new();

  match &field_context.kind {
    FieldKind::MapKey => rule_elements.extend(MAP_KEY_VIOLATION.elements.to_vec()),
    FieldKind::MapValue => rule_elements.extend(MAP_VALUE_VIOLATION.elements.to_vec()),
    FieldKind::RepeatedItem => rule_elements.extend(REPEATED_ITEM_VIOLATION.elements.to_vec()),
    _ => {}
  };

  rule_elements.extend(violation_data.elements.to_vec());

  Violation {
    rule_id: Some(
      custom_rule_id.map_or_else(|| violation_data.name.to_string(), |id| id.to_string()),
    ),
    message: Some(error_message.to_string()),
    for_key: field_context.kind.is_map_key().then_some(true),
    field: Some(FieldPath {
      elements: field_elements,
    }),
    rule: Some(FieldPath {
      elements: rule_elements,
    }),
  }
}

pub(crate) fn new_violation(
  field_context: &FieldContext,
  parent_elements: &[FieldPathElement],
  violation_data: &ViolationData,
  error_message: &str,
) -> Violation {
  create_violation_core(
    None,
    field_context,
    parent_elements,
    violation_data,
    error_message,
  )
}

pub(crate) fn new_violation_with_custom_id(
  rule_id: &str,
  field_context: &FieldContext,
  parent_elements: &[FieldPathElement],
  violation_data: &ViolationData,
  error_message: &str,
) -> Violation {
  create_violation_core(
    Some(rule_id),
    field_context,
    parent_elements,
    violation_data,
    error_message,
  )
}
