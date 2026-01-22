use prelude::proto_types::protovalidate::{Violation, ViolationKind};

use super::*;
use std::sync::LazyLock;

struct CustomValidator;

impl Validator<i32> for CustomValidator {
  type Target = i32;

  fn validate_core<V>(&self, ctx: &mut ValidationCtx, val: Option<&V>) -> ValidatorResult
  where
    V: std::borrow::Borrow<Self::Target> + ?Sized,
  {
    custom_int_validator(ctx, Some(val.unwrap().borrow()))
  }
}

impl Validator<SimpleEnum> for CustomValidator {
  type Target = i32;

  fn validate_core<V>(&self, ctx: &mut ValidationCtx, val: Option<&V>) -> ValidatorResult
  where
    V: std::borrow::Borrow<Self::Target> + ?Sized,
  {
    custom_int_validator(ctx, val.map(|v| v.borrow()))
  }
}

fn test_violation() -> Violation {
  Violation {
    field: None,
    rule: None,
    rule_id: Some("must_be_1".to_string()),
    message: Some("must be 1".to_string()),
    for_key: None,
  }
}

fn custom_int_validator(ctx: &mut ValidationCtx, val: Option<&i32>) -> ValidatorResult {
  let val = val.unwrap();

  if *val == 1 {
    Ok(IsValid::Yes)
  } else {
    ctx.violations.push(ViolationCtx {
      data: test_violation(),
      kind: ViolationKind::Cel,
    });

    Ok(IsValid::No)
  }
}

static CUSTOM_STATIC: LazyLock<CustomValidator> = LazyLock::new(|| CustomValidator);

#[proto_message(no_auto_test)]
struct CustomValidatorsMsg {
  // Tests validators for oneofs/enums/messages for correct type resolution
  #[proto(enum_(SimpleEnum), validate = CustomValidator)]
  custom_enum_validator: i32,
  #[proto(message, validate = CustomValidator)]
  custom_message_validator: Option<CustomTopLevelValidators>,
  #[proto(oneof(tags(1, 2)), validate = CustomValidator)]
  oneof: Option<CustomValidatorOneof>,

  // Tests individual validators in various forms
  #[proto(validate = CustomValidator)]
  custom_struct: i32,
  #[proto(validate = from_fn(custom_int_validator))]
  custom_fn: i32,
  #[proto(validate = *CUSTOM_STATIC)]
  custom_static: i32,
}

#[test]
fn custom_validators() {
  let msg = CustomValidatorsMsg {
    oneof: Some(CustomValidatorOneof::CustomStruct(0)),
    custom_message_validator: Some(CustomTopLevelValidators { id: 0 }),
    ..Default::default()
  };

  let violations = msg.validate_all().unwrap_err().into_violations();

  // 1 for the enum, 1 (+ 4) for the message, 1 (+ 1) for the oneof
  // + 1 each for custom_struct, custom_fn and custom_static
  // = 11
  assert_eq_pretty!(violations.len(), 11);

  for v in violations {
    if v.rule_id() != "cel_rule" {
      assert_eq_pretty!(v, test_violation());
    }
  }
}

impl Validator<CustomValidatorOneof> for CustomValidator {
  type Target = CustomValidatorOneof;

  fn validate_core<V>(&self, ctx: &mut ValidationCtx, val: Option<&V>) -> ValidatorResult
  where
    V: std::borrow::Borrow<Self::Target> + ?Sized,
  {
    custom_oneof_validator(ctx, val.map(|v| v.borrow()))
  }
}

fn custom_oneof_validator(
  ctx: &mut ValidationCtx,
  val: Option<&CustomValidatorOneof>,
) -> ValidatorResult {
  match val.unwrap() {
    CustomValidatorOneof::CustomFn(1) => Ok(IsValid::Yes),
    _ => {
      ctx.violations.push(ViolationCtx {
        data: test_violation(),
        kind: ViolationKind::Cel,
      });

      Ok(IsValid::No)
    }
  }
}

impl Validator<MultipleValidatorsOneof> for CustomValidator {
  type Target = MultipleValidatorsOneof;

  fn validate_core<V>(&self, ctx: &mut ValidationCtx, val: Option<&V>) -> ValidatorResult
  where
    V: std::borrow::Borrow<Self::Target> + ?Sized,
  {
    custom_oneof_validator2(ctx, val.map(|v| v.borrow()))
  }
}

fn custom_oneof_validator2(
  ctx: &mut ValidationCtx,
  val: Option<&MultipleValidatorsOneof>,
) -> ValidatorResult {
  match val.unwrap() {
    MultipleValidatorsOneof::A(1) => Ok(IsValid::Yes),
    _ => {
      ctx.violations.push(ViolationCtx {
        data: test_violation(),
        kind: ViolationKind::Cel,
      });

      Ok(IsValid::No)
    }
  }
}

#[proto_oneof(no_auto_test)]
enum CustomValidatorOneof {
  #[proto(tag = 1, validate = CustomValidator)]
  CustomStruct(i32),
  #[proto(tag = 2, validate = from_fn(custom_int_validator))]
  CustomFn(i32),
  #[proto(tag = 3, validate = *CUSTOM_STATIC)]
  CustomStatic(i32),
}

#[proto_message(no_auto_test)]
struct MultipleValidators {
  #[proto(validate = [ |v| v.const_(1), CustomValidator, from_fn(custom_int_validator), *CUSTOM_STATIC ])]
  id: i32,
  #[proto(oneof(tags(1, 2)), validate = [ CustomValidator, from_fn(custom_oneof_validator2), *CUSTOM_STATIC ])]
  oneof: Option<MultipleValidatorsOneof>,
}

#[proto_oneof(no_auto_test)]
enum MultipleValidatorsOneof {
  #[proto(tag = 1, validate = [ |v| v.const_(1), CustomValidator, from_fn(custom_int_validator), *CUSTOM_STATIC ])]
  A(i32),
  #[proto(tag = 2)]
  B(i32),
}

#[test]
fn multiple_validators() {
  let msg = MultipleValidators {
    id: 0,
    oneof: Some(MultipleValidatorsOneof::A(0)),
  };

  let violations = msg.validate_all().unwrap_err().into_violations();

  // 4 for the `id` field
  // + 3 for the oneof as a field
  // + 4 for the oneof variant
  // = 11
  assert_eq_pretty!(violations.len(), 11);
}

impl Validator<CustomTopLevelValidators> for CustomValidator {
  type Target = CustomTopLevelValidators;

  fn validate_core<V>(&self, ctx: &mut ValidationCtx, val: Option<&V>) -> ValidatorResult
  where
    V: std::borrow::Borrow<Self::Target> + ?Sized,
  {
    custom_top_level_validator(ctx, Some(val.unwrap().borrow()))
  }
}

fn custom_top_level_validator(
  ctx: &mut ValidationCtx,
  val: Option<&CustomTopLevelValidators>,
) -> ValidatorResult {
  let val = val.unwrap();

  if val.id == 1 {
    Ok(IsValid::Yes)
  } else {
    ctx.violations.push(ViolationCtx {
      data: test_violation(),
      kind: ViolationKind::Cel,
    });

    Ok(IsValid::No)
  }
}

static CUSTOM_TOP_LEVEL_STATIC: LazyLock<CustomValidator> = LazyLock::new(|| CustomValidator);

#[proto_message(no_auto_test)]
#[proto(validate = [|v| v.cel(cel_program!(id = "cel_rule", msg = "abc", expr = "this.id == 1")), from_fn(custom_top_level_validator), CustomValidator, *CUSTOM_TOP_LEVEL_STATIC])]
struct CustomTopLevelValidators {
  id: i32,
}

#[test]
fn custom_top_level_validators() {
  let msg = CustomTopLevelValidators::default();

  let violations = msg.validate_all().unwrap_err().into_violations();

  assert_eq_pretty!(violations.len(), 4);
}
