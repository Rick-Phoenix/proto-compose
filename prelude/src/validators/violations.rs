use super::*;

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash)]
pub struct ViolationMeta {
  pub kind: ViolationKind,
  pub field_kind: FieldKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ViolationsAcc {
  metas: Vec<ViolationMeta>,
  violations: Vec<Violation>,
}

pub struct ViolationCtx {
  pub meta: ViolationMeta,
  pub data: Violation,
}

impl ViolationCtx {
  #[must_use]
  pub fn into_violation(self) -> Violation {
    self.into()
  }
}

impl From<ViolationsAcc> for Violations {
  fn from(value: ViolationsAcc) -> Self {
    Self {
      violations: value.violations,
    }
  }
}

impl From<ViolationsAcc> for Vec<Violation> {
  fn from(value: ViolationsAcc) -> Self {
    value.violations
  }
}

impl From<ViolationCtx> for Violation {
  fn from(value: ViolationCtx) -> Self {
    value.data
  }
}

impl IntoIterator for ViolationsAcc {
  type IntoIter = core::iter::Zip<vec::IntoIter<ViolationMeta>, vec::IntoIter<Violation>>;
  type Item = (ViolationMeta, Violation);

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.metas.into_iter().zip(self.violations)
  }
}

impl<'a> IntoIterator for &'a ViolationsAcc {
  type Item = (ViolationMeta, &'a Violation);

  type IntoIter = core::iter::Zip<
    core::iter::Copied<core::slice::Iter<'a, ViolationMeta>>,
    core::slice::Iter<'a, Violation>,
  >;

  fn into_iter(self) -> Self::IntoIter {
    self
      .metas
      .iter()
      .copied()
      .zip(self.violations.iter())
  }
}

impl<'a> IntoIterator for &'a mut ViolationsAcc {
  type Item = (&'a mut ViolationMeta, &'a mut Violation);

  type IntoIter =
    core::iter::Zip<core::slice::IterMut<'a, ViolationMeta>, core::slice::IterMut<'a, Violation>>;

  fn into_iter(self) -> Self::IntoIter {
    self
      .metas
      .iter_mut()
      .zip(self.violations.iter_mut())
  }
}

impl Extend<ViolationCtx> for ViolationsAcc {
  fn extend<T: IntoIterator<Item = ViolationCtx>>(&mut self, iter: T) {
    let iter = iter.into_iter();

    let (lower_bound, _) = iter.size_hint();
    if lower_bound > 0 {
      self.metas.reserve(lower_bound);
      self.violations.reserve(lower_bound);
    }

    for ctx in iter {
      self.metas.push(ctx.meta);
      self.violations.push(ctx.data);
    }
  }
}

impl Extend<(ViolationMeta, Violation)> for ViolationsAcc {
  fn extend<T: IntoIterator<Item = (ViolationMeta, Violation)>>(&mut self, iter: T) {
    let iter = iter.into_iter();

    let (lower_bound, _) = iter.size_hint();
    if lower_bound > 0 {
      self.metas.reserve(lower_bound);
      self.violations.reserve(lower_bound);
    }

    for (meta, data) in iter {
      self.metas.push(meta);
      self.violations.push(data);
    }
  }
}

impl ViolationsAcc {
  pub fn merge(&mut self, other: &mut Self) {
    self.metas.append(&mut other.metas);
    self.violations.append(&mut other.violations);
  }

  #[must_use]
  #[inline]
  pub fn first(&self) -> Option<(ViolationMeta, &Violation)> {
    self
      .metas
      .first()
      .copied()
      .and_then(|k| self.violations.first().map(|v| (k, v)))
  }

  #[must_use]
  #[inline]
  pub fn last(&self) -> Option<(ViolationMeta, &Violation)> {
    self
      .metas
      .last()
      .copied()
      .and_then(|k| self.violations.last().map(|v| (k, v)))
  }

  #[inline]
  pub fn iter(
    &self,
  ) -> core::iter::Zip<
    core::iter::Copied<core::slice::Iter<'_, ViolationMeta>>,
    core::slice::Iter<'_, Violation>,
  > {
    self.into_iter()
  }

  #[inline]
  pub fn iter_mut(
    &mut self,
  ) -> core::iter::Zip<core::slice::IterMut<'_, ViolationMeta>, core::slice::IterMut<'_, Violation>>
  {
    self.into_iter()
  }

  pub fn retain<F>(&mut self, mut f: F)
  where
    F: FnMut(ViolationMeta, &Violation) -> bool,
  {
    let len = self.violations.len();
    let mut keep_count = 0;

    for i in 0..len {
      let should_keep = f(self.metas[i], &self.violations[i]);

      if should_keep {
        if keep_count != i {
          self.metas.swap(keep_count, i);
          self.violations.swap(keep_count, i);
        }
        keep_count += 1;
      }
    }

    self.metas.truncate(keep_count);
    self.violations.truncate(keep_count);
  }

  #[inline(never)]
  #[cold]
  pub fn add_required_oneof_violation(&mut self, parent_elements: &[FieldPathElement]) {
    let violation = create_violation_core(
      Some(ONEOF_REQUIRED_VIOLATION.name.to_string()),
      None,
      parent_elements,
      ONEOF_REQUIRED_VIOLATION,
      "at least one value must be set".into(),
    );

    self.push(ViolationCtx {
      meta: ViolationMeta {
        kind: ViolationKind::RequiredOneof,
        field_kind: FieldKind::default(),
      },
      data: violation,
    });
  }

  #[inline(never)]
  #[cold]
  pub fn add_cel_violation(
    &mut self,
    rule: &CelRule,
    field_context: Option<&FieldContext>,
    parent_elements: &[FieldPathElement],
  ) {
    let violation = create_violation_core(
      Some(rule.id.to_string()),
      field_context,
      parent_elements,
      CEL_VIOLATION,
      rule.message.to_string(),
    );

    self.push(ViolationCtx {
      meta: ViolationMeta {
        kind: ViolationKind::Cel,
        field_kind: field_context
          .map(|fc| fc.field_kind)
          .unwrap_or_default(),
      },
      data: violation,
    });
  }

  #[must_use]
  #[inline]
  pub const fn new() -> Self {
    Self {
      metas: vec![],
      violations: vec![],
    }
  }

  #[inline]
  #[must_use]
  pub fn into_violations(self) -> Violations {
    Violations {
      violations: self.violations,
    }
  }

  #[inline(never)]
  #[cold]
  pub fn push(&mut self, v: ViolationCtx) {
    self.metas.push(v.meta);
    self.violations.push(v.data);
  }

  #[inline]
  #[must_use]
  pub const fn is_empty(&self) -> bool {
    self.violations.is_empty()
  }

  #[inline]
  #[must_use]
  pub const fn len(&self) -> usize {
    self.violations.len()
  }
}

impl Default for ViolationsAcc {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

#[inline(never)]
#[cold]
pub(crate) fn create_violation_core(
  custom_rule_id: Option<String>,
  field_context: Option<&FieldContext>,
  parent_elements: &[FieldPathElement],
  violation_data: ViolationData,
  error_message: String,
) -> Violation {
  let mut field_elements: Option<Vec<FieldPathElement>> = None;
  let mut rule_elements: Vec<FieldPathElement> = Vec::new();
  let mut is_for_key = false;

  // In case of a top level message with CEL violations applied to the message
  // as a whole, there would be no field path
  if let Some(field_context) = field_context {
    let elements = field_elements.get_or_insert_default();

    elements.extend(parent_elements.iter().cloned());

    let current_elem = field_context.as_path_element();

    elements.push(current_elem);

    match &field_context.field_kind {
      FieldKind::MapKey => {
        is_for_key = true;
        rule_elements.extend(MAP_KEYS_VIOLATION.elements_iter());
      }
      FieldKind::MapValue => rule_elements.extend(MAP_VALUES_VIOLATION.elements_iter()),
      FieldKind::RepeatedItem => rule_elements.extend(REPEATED_ITEMS_VIOLATION.elements_iter()),
      _ => {}
    };
  }

  rule_elements.extend(violation_data.elements_iter());

  Violation {
    rule_id: Some(custom_rule_id.unwrap_or_else(|| violation_data.name.to_string())),
    message: Some(error_message),
    for_key: Some(is_for_key),
    field: field_elements.map(|elements| FieldPath { elements }),
    rule: Some(FieldPath {
      elements: rule_elements,
    }),
  }
}
