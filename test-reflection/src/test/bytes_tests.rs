use bytes::Bytes;

use super::*;

macro_rules! bytes {
  ($byt:literal) => {
    $byt.into()
  };
}

#[test]
fn bytes_test() {
  let ipv6: Bytes = bytes!("91bd:9c4e:f03a:e782:cdbc:96ba:7241:9f00");

  let mut msg = BytesRules {
    const_test: bytes!("a"),
    len_test: bytes!("a"),
    min_len_test: bytes!("a"),
    max_len_test: bytes!("a"),
    pattern_test: bytes!("a"),
    prefix_test: bytes!("a"),
    suffix_test: bytes!("a"),
    contains_test: bytes!("a"),
    cel_test: bytes!("a"),
    required_test: Some(bytes!("a")),
    ignore_if_zero_value_test: Some(bytes!("a")),
    ignore_always_test: bytes!("b"),
    ip_test: bytes!("127.0.0.1"),
    ipv4_test: bytes!("127.0.0.1"),
    ipv6_test: ipv6.clone(),
    uuid_test: bytes!("019b8f7c-d933-7be4-8b69-5110ed453a75"),
  };
  let baseline = msg.clone();

  assert!(msg.validate().is_ok(), "basic validation");

  macro_rules! assert_violation {
    ($violation:expr, $error:literal) => {
      assert_violation_id(&msg, $violation, $error);
      msg = baseline.clone();
    };
  }

  msg.const_test = bytes!("b");
  assert_violation!("bytes.const", "const rule");

  msg.len_test = bytes!("ab");
  assert_violation!("bytes.len", "len rule");

  msg.min_len_test = bytes!("");
  assert_violation!("bytes.min_len", "min_len rule");

  msg.max_len_test = bytes!("ab");
  assert_violation!("bytes.max_len", "max_len rule");

  msg.prefix_test = bytes!("b");
  assert_violation!("bytes.prefix", "prefix rule");

  msg.suffix_test = bytes!("b");
  assert_violation!("bytes.suffix", "suffix rule");

  msg.pattern_test = bytes!("b");
  assert_violation!("bytes.pattern", "pattern rule");

  msg.contains_test = bytes!("b");
  assert_violation!("bytes.contains", "contains rule");

  msg.ip_test = ipv6;
  assert!(
    msg.validate().is_ok(),
    "Should support both ipv4 and ipv6 addresses"
  );

  msg.ip_test = bytes!("127.0.0");
  assert_violation!("bytes.ip", "ip rule");

  msg.ipv4_test = bytes!("127.0.0");
  assert_violation!("bytes.ipv4", "ipv4 rule");

  msg.ipv6_test = bytes!("127.0.0");
  assert_violation!("bytes.ipv6", "ipv6 rule");

  msg.uuid_test = bytes!("abcde");
  assert_violation!("bytes.uuid", "uuid rule");

  msg.cel_test = bytes!("b");
  assert_violation!("cel_rule", "cel rule");

  msg.required_test = None;
  assert_violation!("required", "required rule");

  msg.ignore_if_zero_value_test = None;
  assert!(msg.validate().is_ok(), "Should ignore if None");

  msg.ignore_if_zero_value_test = Some(Bytes::default());
  assert!(msg.validate().is_ok(), "Should ignore if empty");
}
