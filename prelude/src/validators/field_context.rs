use super::*;

use proto_types::field_descriptor_proto::Type as ProtoPrimitive;
use proto_types::protovalidate::field_path_element::Subscript;

/// The context for the field being validated.
#[derive(Clone, Debug)]
pub struct FieldContext {
  pub proto_name: &'static str,
  pub tag: i32,
  pub subscript: Option<Subscript>,
  pub map_key_type: Option<ProtoPrimitive>,
  pub map_value_type: Option<ProtoPrimitive>,
  pub field_type: ProtoPrimitive,
  pub field_kind: FieldKind,
}

impl FieldContext {
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

#[derive(Clone, Default, Debug, Copy, PartialEq, Eq, Hash)]
pub enum FieldKind {
  MapKey,
  MapValue,
  RepeatedItem,
  #[default]
  Normal,
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

pub struct ValidationCtx {
  pub field_context: Option<FieldContext>,
  pub parent_elements: Vec<FieldPathElement>,
  pub violations: ViolationsAcc,
  pub fail_fast: bool,
}

impl Default for ValidationCtx {
  #[inline]
  fn default() -> Self {
    Self {
      field_context: None,
      parent_elements: vec![],
      violations: ViolationsAcc::new(),
      fail_fast: true,
    }
  }
}

macro_rules! violation_helpers {
  ($($name:ident),*) => {
    paste::paste! {
      $(
        pub(crate) fn [< add_ $name _violation >](&mut self, kind: [< $name:camel Violation >], error_message: CowStr) -> ValidatorResult {
          self.add_violation(ViolationKind::[< $name:camel >](kind), error_message)
        }
      )*
    }
  };
}

type CowStr<'a> = Cow<'a, str>;

impl ValidationCtx {
  #[inline]
  pub fn reset_field_context(&mut self) {
    self.field_context = None;
  }

  #[inline]
  pub fn with_field_context(&mut self, field_context: FieldContext) -> &mut Self {
    self.field_context = Some(field_context);
    self
  }

  violation_helpers!(
    any, bool, bytes, duration, string, timestamp, enum, field_mask, map, repeated
  );

  #[inline]
  pub fn add_violation(
    &mut self,
    kind: ViolationKind,
    error_message: impl Into<String>,
  ) -> ValidatorResult {
    let violation = create_violation_core(
      None,
      self.field_context.as_ref(),
      &self.parent_elements,
      kind.data(),
      error_message.into(),
    );

    self.violations.push(ViolationCtx {
      meta: ViolationMeta {
        kind,
        field_kind: self.field_kind(),
      },
      data: violation,
    });

    if self.fail_fast {
      Err(FailFast)
    } else {
      Ok(IsValid::No)
    }
  }

  fn field_kind(&self) -> FieldKind {
    self
      .field_context
      .as_ref()
      .map(|fc| fc.field_kind)
      .unwrap_or_default()
  }

  #[inline]
  pub fn add_violation_with_custom_id(
    &mut self,
    rule_id: &str,
    kind: ViolationKind,
    error_message: impl Into<String>,
  ) -> ValidatorResult {
    let violation = new_violation_with_custom_id(
      rule_id,
      self.field_context.as_ref(),
      &self.parent_elements,
      kind.data(),
      error_message.into(),
    );

    self.violations.push(ViolationCtx {
      data: violation,
      meta: ViolationMeta {
        kind,
        field_kind: self.field_kind(),
      },
    });

    if self.fail_fast {
      Err(FailFast)
    } else {
      Ok(IsValid::No)
    }
  }

  #[inline]
  pub fn add_cel_violation(&mut self, rule: &CelRule) -> ValidatorResult {
    self
      .violations
      .add_cel_violation(rule, self.field_context.as_ref(), &self.parent_elements);

    if self.fail_fast {
      Err(FailFast)
    } else {
      Ok(IsValid::No)
    }
  }

  #[inline]
  pub fn add_required_oneof_violation(&mut self) -> ValidatorResult {
    self
      .violations
      .add_required_oneof_violation(&self.parent_elements);

    if self.fail_fast {
      Err(FailFast)
    } else {
      Ok(IsValid::No)
    }
  }

  #[inline]
  pub fn add_required_violation(&mut self) -> ValidatorResult {
    self.add_violation(ViolationKind::Required, Cow::Borrowed("is required"))
  }

  #[cfg(feature = "cel")]
  pub(crate) fn add_cel_error_violation(&mut self, error: CelError) -> ValidatorResult {
    self.violations.push(ViolationCtx {
      meta: ViolationMeta {
        kind: ViolationKind::Cel,
        field_kind: self.field_kind(),
      },
      data: error.into_violation(self.field_context.as_ref(), &self.parent_elements),
    });

    if self.fail_fast {
      Err(FailFast)
    } else {
      Ok(IsValid::No)
    }
  }
}
