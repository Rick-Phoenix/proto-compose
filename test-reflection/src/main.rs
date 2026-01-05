mod proto {
  include!(concat!(env!("OUT_DIR"), "/reflection.v1.rs"));
}

fn main() {
  println!("Hello, world!");
}

#[cfg(test)]
mod test {
  use prelude::ValidatedMessage;

  #[allow(unused)]
  pub(crate) fn get_rules_ids<T: ValidatedMessage>(msg: &T) -> Vec<String> {
    let violations = msg.validate().unwrap_err();

    violations
      .into_iter()
      .map(|v| v.rule_id().to_string())
      .collect()
  }

  #[track_caller]
  pub(crate) fn assert_violation_id(msg: &impl ValidatedMessage, expected: &str, error: &str) {
    let violations = msg.validate().unwrap_err();

    assert_eq!(violations.len(), 1, "Expected a single violation");
    assert_eq!(violations.first().unwrap().rule_id(), expected, "{error}");
  }

  macro_rules! num {
    ($num:literal, finite_test) => {
      ($num as f32).into()
    };
    ($num:literal) => {
      $num
    };
  }

  mod bool_tests;
  mod bytes_tests;
  mod duration_tests;
  mod numeric_tests;
  mod string_tests;
  mod timestamp_tests;
}
