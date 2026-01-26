use core::{
  iter::{Copied, Zip},
  slice,
};

use super::*;

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash)]
pub struct ViolationMeta {
  pub kind: ViolationKind,
  pub field_kind: FieldKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ViolationErrors {
  metas: Vec<ViolationMeta>,
  violations: Vec<Violation>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ViolationCtx {
  pub meta: ViolationMeta,
  pub data: Violation,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub struct ViolationCtxRef<'a> {
  pub meta: ViolationMeta,
  pub data: &'a Violation,
}

impl ViolationCtxRef<'_> {
  #[must_use]
  #[inline]
  pub fn into_owned(self) -> ViolationCtx {
    ViolationCtx {
      meta: self.meta,
      data: self.data.clone(),
    }
  }

  #[must_use]
  #[inline]
  pub fn into_violation(self) -> Violation {
    self.into_owned().into_violation()
  }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ViolationCtxMut<'a> {
  pub meta: &'a mut ViolationMeta,
  pub data: &'a mut Violation,
}

impl ViolationCtxMut<'_> {
  #[must_use]
  #[inline]
  pub fn into_owned(&self) -> ViolationCtx {
    ViolationCtx {
      meta: *self.meta,
      data: self.data.clone(),
    }
  }

  #[must_use]
  #[inline]
  pub fn into_violation(&self) -> Violation {
    self.into_owned().into_violation()
  }
}

#[derive(Clone, Debug)]
pub struct IntoIter {
  iter: core::iter::Zip<alloc::vec::IntoIter<ViolationMeta>, alloc::vec::IntoIter<Violation>>,
}

impl Iterator for IntoIter {
  type Item = ViolationCtx;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    self
      .iter
      .next()
      .map(|(meta, data)| ViolationCtx { meta, data })
  }

  #[inline]
  fn size_hint(&self) -> (usize, Option<usize>) {
    self.iter.size_hint()
  }
}

impl ExactSizeIterator for IntoIter {
  #[inline]
  fn len(&self) -> usize {
    self.iter.len()
  }
}

#[derive(Clone, Debug)]
pub struct Iter<'a> {
  iter: Zip<Copied<slice::Iter<'a, ViolationMeta>>, slice::Iter<'a, Violation>>,
}

impl<'a> Iterator for Iter<'a> {
  type Item = ViolationCtxRef<'a>;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    self
      .iter
      .next()
      .map(|(meta, data)| ViolationCtxRef { meta, data })
  }

  #[inline]
  fn size_hint(&self) -> (usize, Option<usize>) {
    self.iter.size_hint()
  }
}

impl ExactSizeIterator for Iter<'_> {
  #[inline]
  fn len(&self) -> usize {
    self.iter.len()
  }
}

#[derive(Debug)]
pub struct IterMut<'a> {
  iter: Zip<core::slice::IterMut<'a, ViolationMeta>, core::slice::IterMut<'a, Violation>>,
}

impl<'a> Iterator for IterMut<'a> {
  type Item = ViolationCtxMut<'a>;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    self
      .iter
      .next()
      .map(|(meta, data)| ViolationCtxMut { meta, data })
  }

  #[inline]
  fn size_hint(&self) -> (usize, Option<usize>) {
    self.iter.size_hint()
  }
}

impl ExactSizeIterator for IterMut<'_> {
  #[inline]
  fn len(&self) -> usize {
    self.iter.len()
  }
}

impl ViolationCtx {
  #[must_use]
  #[inline]
  pub fn into_violation(self) -> Violation {
    self.into()
  }
}

impl From<ViolationErrors> for Violations {
  #[inline]
  fn from(value: ViolationErrors) -> Self {
    Self {
      violations: value.violations,
    }
  }
}

impl From<ViolationErrors> for Vec<Violation> {
  #[inline]
  fn from(value: ViolationErrors) -> Self {
    value.violations
  }
}

impl From<ViolationCtx> for Violation {
  #[inline]
  fn from(value: ViolationCtx) -> Self {
    value.data
  }
}

impl IntoIterator for ViolationErrors {
  type IntoIter = IntoIter;
  type Item = ViolationCtx;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    IntoIter {
      iter: self.metas.into_iter().zip(self.violations),
    }
  }
}

impl<'a> IntoIterator for &'a ViolationErrors {
  type Item = ViolationCtxRef<'a>;

  type IntoIter = Iter<'a>;

  fn into_iter(self) -> Self::IntoIter {
    Iter {
      iter: self
        .metas
        .iter()
        .copied()
        .zip(self.violations.iter()),
    }
  }
}

impl<'a> IntoIterator for &'a mut ViolationErrors {
  type Item = ViolationCtxMut<'a>;

  type IntoIter = IterMut<'a>;

  fn into_iter(self) -> Self::IntoIter {
    IterMut {
      iter: self
        .metas
        .iter_mut()
        .zip(self.violations.iter_mut()),
    }
  }
}

impl Extend<ViolationCtx> for ViolationErrors {
  fn extend<T: IntoIterator<Item = ViolationCtx>>(&mut self, iter: T) {
    let iter = iter.into_iter();

    let (lower_bound, _) = iter.size_hint();
    if lower_bound > 0 {
      self.metas.reserve(lower_bound);
      self.violations.reserve(lower_bound);
    }

    for v in iter {
      self.metas.push(v.meta);
      self.violations.push(v.data);
    }
  }
}

impl ViolationErrors {
  #[inline]
  pub fn merge(&mut self, other: &mut Self) {
    self.metas.append(&mut other.metas);
    self.violations.append(&mut other.violations);
  }

  #[inline]
  #[must_use]
  pub fn iter(&self) -> Iter<'_> {
    self.into_iter()
  }

  #[inline]
  pub fn iter_mut(&mut self) -> IterMut<'_> {
    self.into_iter()
  }

  pub fn retain<F>(&mut self, mut f: F)
  where
    F: FnMut(ViolationCtxRef) -> bool,
  {
    let len = self.violations.len();
    let mut keep_count = 0;

    for i in 0..len {
      let should_keep = f(ViolationCtxRef {
        meta: self.metas[i],
        data: &self.violations[i],
      });

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

  #[inline]
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

impl Default for ViolationErrors {
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

  // In case of a top level validator there would be no field path
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
