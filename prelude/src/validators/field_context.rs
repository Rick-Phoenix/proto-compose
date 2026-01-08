use std::ops::{Deref, DerefMut};

use super::*;

use proto_types::field_descriptor_proto::Type as ProtoPrimitive;
use proto_types::protovalidate::field_path_element::Subscript;
use smallvec::SmallVec;

/// The context for the field being validated.
#[derive(Clone, Debug)]
pub struct FieldContext<'a> {
  pub proto_name: &'a str,
  pub tag: i32,
  pub subscript: Option<Subscript>,
  pub map_key_type: Option<ProtoPrimitive>,
  pub map_value_type: Option<ProtoPrimitive>,
  pub field_type: ProtoPrimitive,
  pub field_kind: FieldKind,
}

impl FieldContext<'_> {
  #[must_use]
  pub fn as_path_element(&self) -> FieldPathElement {
    FieldPathElement {
      field_number: Some(self.tag),
      field_name: Some(self.proto_name.to_string()),
      field_type: Some(self.field_type as i32),
      key_type: self.map_key_type.map(|t| t as i32),
      value_type: self.map_value_type.map(|t| t as i32),
      subscript: self.subscript.clone(),
    }
  }
}

#[derive(Clone, Default, Debug, Copy, PartialEq, Eq)]
pub enum FieldKind {
  Map,
  MapKey,
  MapValue,
  Repeated,
  RepeatedItem,
  #[default]
  Single,
}

impl FieldKind {
  #[must_use]
  pub const fn is_map_key(&self) -> bool {
    matches!(self, Self::MapKey)
  }

  #[must_use]
  pub const fn is_map_value(&self) -> bool {
    matches!(self, Self::MapValue)
  }

  #[must_use]
  pub const fn is_repeated_item(&self) -> bool {
    matches!(self, Self::RepeatedItem)
  }
}

pub struct ValidationCtx<'a> {
  pub field_context: FieldContext<'a>,
  pub parent_elements: &'a mut Vec<FieldPathElement>,
  pub violations: &'a mut ViolationsAcc,
}

impl ValidationCtx<'_> {
  #[inline]
  pub fn add_violation(&mut self, violation_data: &ViolationData, error_message: &str) {
    let violation = new_violation(
      &self.field_context,
      self.parent_elements,
      violation_data,
      error_message,
    );

    self.violations.push(violation);
  }

  #[inline]
  pub fn add_violation_with_custom_id(
    &mut self,
    rule_id: &str,
    violation_data: &ViolationData,
    error_message: &str,
  ) {
    let violation = new_violation_with_custom_id(
      rule_id,
      Some(&self.field_context),
      self.parent_elements,
      violation_data,
      error_message,
    );

    self.violations.push(violation);
  }

  #[inline]
  pub fn add_cel_violation(&mut self, rule: &CelRule) {
    self
      .violations
      .add_cel_violation(rule, Some(&self.field_context), self.parent_elements);
  }

  #[inline]
  pub fn add_required_oneof_violation(&mut self) {
    self
      .violations
      .add_required_oneof_violation(self.parent_elements);
  }

  #[inline]
  pub fn add_required_violation(&mut self) {
    self.add_violation(&REQUIRED_VIOLATION, "is required")
  }
}

pub struct ViolationsAcc {
  inner: SmallVec<[Violation; 1]>,
}

impl IntoIterator for ViolationsAcc {
  type IntoIter = smallvec::IntoIter<[Violation; 1]>;
  type Item = Violation;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.inner.into_iter()
  }
}

impl Deref for ViolationsAcc {
  type Target = [Violation];

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

impl DerefMut for ViolationsAcc {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.inner
  }
}

impl<'a> IntoIterator for &'a ViolationsAcc {
  type IntoIter = std::slice::Iter<'a, Violation>;
  type Item = &'a Violation;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.inner.iter()
  }
}

impl Extend<Violation> for ViolationsAcc {
  #[inline]
  fn extend<T: IntoIterator<Item = Violation>>(&mut self, iter: T) {
    self.inner.extend(iter);
  }
}

impl ViolationsAcc {
  #[inline]
  pub fn iter(&self) -> std::slice::Iter<'_, Violation> {
    self.inner.iter()
  }
  #[inline]
  pub fn add_required_oneof_violation(&mut self, parent_elements: &[FieldPathElement]) {
    let violation = new_violation_with_custom_id(
      ONEOF_REQUIRED_VIOLATION.name,
      None,
      parent_elements,
      &ONEOF_REQUIRED_VIOLATION,
      "at least one value must be set",
    );

    self.inner.push(violation);
  }

  #[inline]
  pub fn add_cel_violation(
    &mut self,
    rule: &CelRule,
    field_context: Option<&FieldContext>,
    parent_elements: &[FieldPathElement],
  ) {
    let violation = new_violation_with_custom_id(
      &rule.id,
      field_context,
      parent_elements,
      &CEL_VIOLATION,
      &rule.message,
    );

    self.push(violation);
  }

  #[must_use]
  pub fn new() -> Self {
    Self {
      inner: SmallVec::new(),
    }
  }

  #[inline]
  #[must_use]
  pub fn to_vec(self) -> Violations {
    Violations {
      violations: self.inner.to_vec(),
    }
  }

  #[inline]
  pub fn push(&mut self, v: Violation) {
    self.inner.push(v);
  }

  #[inline]
  #[must_use]
  pub fn is_empty(&self) -> bool {
    self.inner.is_empty()
  }

  pub fn into_result(self) -> Result<(), Vec<Violation>> {
    if self.inner.is_empty() {
      Ok(())
    } else {
      Err(self.inner.into_vec())
    }
  }
}

impl Default for ViolationsAcc {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

pub(crate) fn create_violation_core(
  custom_rule_id: Option<&str>,
  field_context: Option<&FieldContext>,
  parent_elements: &[FieldPathElement],
  violation_data: &ViolationData,
  error_message: &str,
) -> Violation {
  let mut field_elements: Option<Vec<FieldPathElement>> = None;
  let mut rule_elements: Vec<FieldPathElement> = Vec::new();
  let mut is_for_key = false;

  // In case of a top level message with CEL violations applied to the message
  // as a whole, there would be no field path
  if let Some(field_context) = field_context {
    let elements = field_elements.get_or_insert_default();

    elements.extend(parent_elements.iter().cloned());

    let current_elem = FieldPathElement {
      field_type: Some(field_context.field_type as i32),
      field_name: Some(field_context.proto_name.to_string()),
      key_type: field_context.map_key_type.map(|t| t as i32),
      value_type: field_context.map_value_type.map(|t| t as i32),
      field_number: Some(field_context.tag),
      subscript: field_context.subscript.clone(),
    };

    elements.push(current_elem);

    match &field_context.field_kind {
      FieldKind::MapKey => {
        is_for_key = true;
        rule_elements.extend(MAP_KEY_VIOLATION.elements.to_vec());
      }
      FieldKind::MapValue => rule_elements.extend(MAP_VALUE_VIOLATION.elements.to_vec()),
      FieldKind::RepeatedItem => rule_elements.extend(REPEATED_ITEM_VIOLATION.elements.to_vec()),
      _ => {}
    };
  }

  rule_elements.extend(violation_data.elements.to_vec());

  Violation {
    rule_id: Some(
      custom_rule_id.map_or_else(|| violation_data.name.to_string(), |id| id.to_string()),
    ),
    message: Some(error_message.to_string()),
    for_key: Some(is_for_key),
    field: field_elements.map(|elements| FieldPath { elements }),
    rule: Some(FieldPath {
      elements: rule_elements,
    }),
  }
}

#[inline]
pub(crate) fn new_violation(
  field_context: &FieldContext,
  parent_elements: &[FieldPathElement],
  violation_data: &ViolationData,
  error_message: &str,
) -> Violation {
  create_violation_core(
    None,
    Some(field_context),
    parent_elements,
    violation_data,
    error_message,
  )
}

#[inline]
pub(crate) fn new_violation_with_custom_id(
  rule_id: &str,
  field_context: Option<&FieldContext>,
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
