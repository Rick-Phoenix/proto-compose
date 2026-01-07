#![allow(clippy::clone_on_copy)]

use super::*;

macro_rules! num {
  ($num:literal, finite_test) => {
    ($num as f32).into()
  };
  ($num:literal) => {
    $num
  };
}

macro_rules! test_numeric {
  ($name:ident $(, $finite_test:ident, $float_type:ty)?) => {
    paste::paste! {
      #[test]
      #[allow(unused_assignments)]
      fn [< test_ $name:snake >]() {
        let mut msg = [< $name:camel Rules >] {
          required_test: Some(num!(1 $(, $finite_test)?)),
          lt_test: num!(0 $(, $finite_test)?),
          lte_test: num!(1 $(, $finite_test)?),
          gt_test: num!(2 $(, $finite_test)?),
          gte_test: num!(1 $(, $finite_test)?),
          in_test: num!(1 $(, $finite_test)?),
          not_in_test: num!(2 $(, $finite_test)?),
          cel_test: num!(1 $(, $finite_test)?),
          ignore_if_zero_value_test: None,
          ignore_always_test: num!(3 $(, $finite_test)?),
          const_test: num!(1 $(, $finite_test)?),
          $($finite_test: 1.0)?
        };
        let baseline = msg.clone();

        // This implicitly tests `ignore_always` too
        assert!(msg.validate().is_ok());

        macro_rules! assert_violation {
          ($violation:expr, $error:literal) => {
            assert_violation_id(&msg, $violation, $error);
            msg = baseline.clone();
          };
        }

        macro_rules! rule {
          ($rule:literal) => (concat!(stringify!([< $name:lower >]), ".", $rule))
        }

        msg.required_test = None;
        assert_violation!("required", "Must be Some");

        msg.lt_test = num!(1 $(, $finite_test)?);
        assert_violation!(rule!("lt"), "lt rule");

        msg.lt_test = num!(2 $(, $finite_test)?);
        assert_violation!(rule!("lt"), "lt rule");

        msg.lte_test = num!(2 $(, $finite_test)?);
        assert_violation!(rule!("lte"), "lte rule");

        msg.gt_test = num!(1 $(, $finite_test)?);
        assert_violation!(rule!("gt"), "gt rule");

        msg.gt_test = num!(0 $(, $finite_test)?);
        assert_violation!(rule!("gt"), "gt rule");

        msg.gte_test = num!(0 $(, $finite_test)?);
        assert_violation!(rule!("gte"), "gte rule");

        msg.in_test = num!(2 $(, $finite_test)?);
        assert_violation!(rule!("in"), "in rule");

        msg.not_in_test = num!(1 $(, $finite_test)?);
        assert_violation!(rule!("not_in"), "not_in rule");

        msg.cel_test = num!(0 $(, $finite_test)?);
        assert_violation!("cel_rule", "cel rule");

        msg.ignore_if_zero_value_test = Some(num!(0 $(, $finite_test)?));
        assert!(msg.validate().is_ok(), "Must ignore if zero value");

        msg.ignore_if_zero_value_test = Some(num!(2 $(, $finite_test)?));
        assert_violation!(rule!("const"), "Must not ignore if not zero value");

        $(
          msg.finite_test = $float_type::NAN;
          assert_violation!(rule!("finite"), "finite rule");

          msg.finite_test = $float_type::INFINITY;
          assert_violation!(rule!("finite"), "finite rule");
        )?
      }
    }
  };
}

test_numeric!(Int64);
test_numeric!(Sint64);
test_numeric!(Sfixed64);
test_numeric!(Int32);
test_numeric!(Sint32);
test_numeric!(Sfixed32);
test_numeric!(Uint64);
test_numeric!(Fixed64);
test_numeric!(Uint32);
test_numeric!(Fixed32);
test_numeric!(Float, finite_test, f32);
test_numeric!(Double, finite_test, f64);
