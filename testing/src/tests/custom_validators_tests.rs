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
    custom_validator(ctx, Some(val.unwrap().borrow()))
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

fn custom_validator(ctx: &mut ValidationCtx, val: Option<&i32>) -> ValidatorResult {
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
  #[proto(validate = CustomValidator)]
  custom_struct: i32,
  #[proto(validate = from_fn(custom_validator))]
  custom_fn: i32,
  #[proto(validate = *CUSTOM_STATIC)]
  custom_static: i32,
  #[proto(oneof(tags(1, 2)))]
  oneof: Option<CustomValidatorOneof>,
}

#[proto_oneof(no_auto_test)]
enum CustomValidatorOneof {
  #[proto(tag = 1, validate = CustomValidator)]
  CustomStruct(i32),
  #[proto(tag = 2, validate = from_fn(custom_validator))]
  CustomFn(i32),
  #[proto(tag = 3, validate = *CUSTOM_STATIC)]
  CustomStatic(i32),
}

#[test]
fn custom_validators() {
  let msg = CustomValidatorsMsg {
    oneof: Some(CustomValidatorOneof::CustomStruct(0)),
    ..Default::default()
  };

  let violations = msg.validate_all().unwrap_err().into_violations();

  assert_eq_pretty!(violations.len(), 4);

  for v in violations {
    assert_eq_pretty!(v, test_violation());
  }
}

#[proto_message(no_auto_test)]
struct MultipleValidators {
  #[proto(validate = [ |v| v.const_(1), CustomValidator, from_fn(custom_validator), *CUSTOM_STATIC ])]
  id: i32,
  #[proto(oneof(tags(1, 2)))]
  oneof: Option<MultipleValidatorsOneof>,
}

#[proto_oneof(no_auto_test)]
enum MultipleValidatorsOneof {
  #[proto(tag = 1, validate = [ |v| v.const_(1), CustomValidator, from_fn(custom_validator), *CUSTOM_STATIC ])]
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

  assert_eq_pretty!(violations.len(), 8);
}

struct CustomMsgValidator;

impl Validator<CustomTopLevelValidators> for CustomMsgValidator {
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

static CUSTOM_TOP_LEVEL_STATIC: LazyLock<CustomMsgValidator> = LazyLock::new(|| CustomMsgValidator);

#[proto_message(no_auto_test)]
#[proto(validate = [|v| v.cel(cel_program!(id = "abc", msg = "abc", expr = "this.id == 1")), from_fn(custom_top_level_validator), CustomMsgValidator, *CUSTOM_TOP_LEVEL_STATIC])]
struct CustomTopLevelValidators {
  id: i32,
}

#[test]
fn custom_top_level_validators() {
  let msg = CustomTopLevelValidators::default();

  let violations = msg.validate_all().unwrap_err().into_violations();

  assert_eq_pretty!(violations.len(), 4);
}
