use super::*;

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub struct IntWrapper(i32);

impl From<i32> for IntWrapper {
  fn from(value: i32) -> Self {
    Self(value)
  }
}

impl From<IntWrapper> for i32 {
  fn from(value: IntWrapper) -> Self {
    value.0
  }
}

// This implicitly checks the automatic conversion working
#[proto_oneof(proxied)]
#[proto(skip_checks(all))]
#[derive(PartialEq)]
pub enum ProxiedOneofWithDefault {
  #[proto(tag = 1)]
  A(String),
  #[proto(tag = 2, int32)]
  B(IntWrapper),
}

impl Default for ProxiedOneofWithDefaultProto {
  fn default() -> Self {
    Self::B(5)
  }
}

#[proto_message(proxied)]
#[proto(skip_checks(all))]
pub struct WithProxiedDefaultOneof {
  #[proto(oneof(proxied, default, tags(1, 2)))]
  field: ProxiedOneofWithDefault,
}

#[test]
fn proxied_oneof_with_default() {
  let msg = WithProxiedDefaultOneofProto::default();

  // The conversion should have used the default impl
  let converted: WithProxiedDefaultOneof = msg.into();

  assert_eq_pretty!(converted.field, ProxiedOneofWithDefault::B(IntWrapper(5)));
}

// This should compile because using a oneof not wrapped with `Option` should be allowed
// if a custom conversion impl is provided
#[proto_message(proxied)]
#[proto(skip_checks(all))]
pub struct DefaultOneofWithCustomImpl {
  #[proto(oneof(proxied, tags(1, 2)))]
  #[proto(from_proto = |_| ProxiedOneofWithDefault::B(IntWrapper(0)), into_proto = |_| Some(ProxiedOneofWithDefaultProto::default()))]
  oneof: ProxiedOneofWithDefault,
}

#[proto_oneof]
#[proto(skip_checks(all))]
pub enum DirectOneofWithDefault {
  #[proto(tag = 1)]
  A(String),
  #[proto(tag = 2)]
  B(i32),
}

impl Default for DirectOneofWithDefault {
  fn default() -> Self {
    Self::B(1)
  }
}

#[proto_message(proxied)]
#[proto(skip_checks(all))]
pub struct WithDirectDefaultOneof {
  #[proto(oneof(default, tags(1, 2)))]
  field: DirectOneofWithDefault,
}

#[test]
fn direct_oneof_with_default() {
  let msg = WithDirectDefaultOneofProto::default();

  let proxy = msg.into_proxy();

  assert_eq_pretty!(proxy.field, DirectOneofWithDefault::default())
}

#[proto_oneof(proxied)]
#[proto(skip_checks(all))]
#[derive(PartialEq)]
pub enum OneofCustomFieldConversions {
  #[proto(tag = 1, string, from_proto = |_| "from_proto".into(), into_proto = |_| "into_proto".to_string())]
  A(Arc<str>),
  #[proto(tag = 2, int32, from_proto = |_| IntWrapper(0), into_proto = |_| 1)]
  B(IntWrapper),
}

#[proto_message(proxied)]
#[proto(skip_checks(all))]
pub struct WithOneofWithCustomFieldConversions {
  #[proto(oneof(proxied, tags(1, 2)))]
  field: Option<OneofCustomFieldConversions>,
}

#[test]
fn oneof_with_custom_field_conversions() {
  let proxy = WithOneofWithCustomFieldConversions {
    field: Some(OneofCustomFieldConversions::A(Default::default())),
  };

  let msg = proxy.into_message();

  assert_eq_pretty!(
    msg.field,
    Some(OneofCustomFieldConversionsProto::A("into_proto".into()))
  );

  let new_proxy = msg.into_proxy();

  assert_eq_pretty!(
    new_proxy.field,
    Some(OneofCustomFieldConversions::A("from_proto".into()))
  );
}

#[proto_oneof(proxied)]
#[proto(skip_checks(all))]
#[derive(PartialEq)]
pub enum OneofFieldIntoProtoOnly {
  #[proto(tag = 1, string, into_proto = |_| "into_proto".into())]
  A(Arc<str>),
  #[proto(tag = 2, int32)]
  B(i32),
}

#[test]
fn oneof_field_into_proto_only() {
  let proxy = OneofFieldIntoProtoOnly::A("abc".into());
  let oneof = proxy.into_oneof();

  assert_eq_pretty!(oneof, OneofFieldIntoProtoOnlyProto::A("into_proto".into()));
}

#[proto_oneof(proxied)]
#[proto(skip_checks(all))]
#[derive(PartialEq)]
pub enum OneofFieldFromProtoOnly {
  #[proto(tag = 1, string)]
  A(String),
  #[proto(tag = 2, int32, from_proto = |_| IntWrapper(1))]
  B(IntWrapper),
}

#[test]
fn oneof_field_from_proto_only() {
  let oneof = OneofFieldFromProtoOnlyProto::B(0);
  let proxy: OneofFieldFromProtoOnly = oneof.into();

  assert_eq_pretty!(proxy, OneofFieldFromProtoOnly::B(IntWrapper(1)));
}

#[proto_oneof(proxied)]
#[proto(skip_checks(all))]
#[derive(PartialEq)]
#[proto(from_proto = |_| OneofCustomConversions::A("from_proto".to_string()))]
#[proto(into_proto = |_| OneofCustomConversionsProto::A("into_proto".to_string()))]
pub enum OneofCustomConversions {
  #[proto(tag = 1, string)]
  A(String),
  #[allow(unused)]
  #[proto(tag = 2, int32)]
  B(IntWrapper),
}

#[test]
fn oneof_custom_conversions() {
  let mut proxy = OneofCustomConversions::A("abc".to_string());

  let oneof = proxy.into_oneof();

  assert_eq_pretty!(
    oneof,
    OneofCustomConversionsProto::A("into_proto".to_string())
  );

  proxy = oneof.into_proxy();

  assert_eq_pretty!(proxy, OneofCustomConversions::A("from_proto".to_string()));
}

#[proto_oneof(proxied)]
#[proto(skip_checks(all))]
#[derive(PartialEq)]
#[proto(into_proto = |_| OneofIntoProtoOnlyProto::A("into_proto".to_string()))]
pub enum OneofIntoProtoOnly {
  #[proto(tag = 1, string)]
  A(String),
  #[allow(unused)]
  #[proto(tag = 2, int32)]
  B(IntWrapper),
}

#[test]
fn oneof_into_proto_only() {
  let proxy = OneofIntoProtoOnly::A("abc".to_string());

  let oneof = proxy.into_oneof();

  assert_eq_pretty!(oneof, OneofIntoProtoOnlyProto::A("into_proto".to_string()));
}

#[proto_oneof(proxied)]
#[proto(skip_checks(all))]
#[derive(PartialEq)]
#[proto(from_proto = |_| OneofFromProtoOnly::A("from_proto".to_string()))]
pub enum OneofFromProtoOnly {
  #[proto(tag = 1, string)]
  A(String),
  #[allow(unused)]
  #[proto(tag = 2, int32)]
  B(IntWrapper),
}

#[test]
fn oneof_from_proto_only() {
  let oneof = OneofFromProtoOnlyProto::A("abc".to_string());

  let proxy = oneof.into_proxy();

  assert_eq_pretty!(proxy, OneofFromProtoOnly::A("from_proto".to_string()));
}

#[proto_message(proxied)]
#[proto(skip_checks(all))]
// Implicitly checks proto derives working
#[proto(derive(Copy))]
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct ProxiedMessage1 {
  #[proto(int32, validate = |v| v.const_(1))]
  id: IntWrapper,
}

#[test]
fn conversion_helpers() {
  let mut proxy = ProxiedMessage1::default();
  let mut msg = ProxiedMessage1Proto::default();

  assert_eq_pretty!(proxy, msg.into_proxy());
  assert_eq_pretty!(msg, proxy.into_message());

  assert!(proxy.into_validated_message().is_err());
  assert!(ProxiedMessage1::from_validated_message(msg).is_err());

  proxy.id = IntWrapper(1);
  msg.id = 1;

  assert_eq_pretty!(proxy.into_validated_message().unwrap(), msg);
  assert_eq_pretty!(ProxiedMessage1::from_validated_message(msg).unwrap(), proxy);
}

// This just checks if the proxy is working
#[proto_message(proxied)]
#[proto(skip_checks(all))]
// Checks if attr forwarding works
#[proto(attr(derive(Copy)))]
pub struct ProxiedMessage2 {
  #[proto(message(proxied))]
  msg: Option<ProxiedMessage1>,
}

#[proto_message(proxied)]
#[proto(skip_checks(all))]
pub struct MessageFieldCustomConversions {
  #[proto(int32)]
  #[proto(from_proto = |_| IntWrapper(1))]
  #[proto(into_proto = |_| 2)]
  id: IntWrapper,
}

#[test]
fn message_field_custom_conversions() {
  let mut msg = MessageFieldCustomConversionsProto::default();

  let proxy: MessageFieldCustomConversions = msg.into();

  assert_eq_pretty!(proxy.id.0, 1);

  msg = proxy.into_message();

  assert_eq_pretty!(msg.id, 2);
}

#[proto_message(proxied)]
#[proto(skip_checks(all))]
pub struct MessageFieldCustomFromProtoOnly {
  #[proto(int32)]
  #[proto(from_proto = |_| IntWrapper(1))]
  id: IntWrapper,
}

#[test]
fn message_field_custom_from_proto_only() {
  let msg = MessageFieldCustomFromProtoOnlyProto::default();

  let proxy: MessageFieldCustomFromProtoOnly = msg.into();

  assert_eq_pretty!(proxy.id.0, 1);
}

#[proto_message(proxied)]
#[proto(skip_checks(all))]
pub struct MessageFieldCustomIntoProtoOnly {
  #[proto(int32)]
  #[proto(into_proto = |_| 2)]
  id: IntWrapper,
}

#[test]
fn message_field_custom_into_proto_only() {
  let proxy = MessageFieldCustomIntoProtoOnly { id: IntWrapper(0) };

  let msg = proxy.into_message();

  assert_eq_pretty!(msg.id, 2);
}

#[proto_message(proxied)]
#[proto(skip_checks(all))]
#[proto(from_proto = |_| MessageCustomConversions { id: IntWrapper(1) })]
#[proto(into_proto = |_| MessageCustomConversionsProto { id: 2 })]
pub struct MessageCustomConversions {
  #[proto(int32)]
  id: IntWrapper,
}

#[test]
fn message_custom_conversions() {
  let mut msg = MessageCustomConversionsProto::default();

  let proxy: MessageCustomConversions = msg.into();

  assert_eq_pretty!(proxy.id.0, 1);

  msg = proxy.into_message();

  assert_eq_pretty!(msg.id, 2);
}

#[proto_message(proxied)]
#[proto(skip_checks(all))]
#[proto(from_proto = |_| MessageCustomFromProtoOnly { id: IntWrapper(1) })]
pub struct MessageCustomFromProtoOnly {
  #[proto(int32)]
  id: IntWrapper,
}

#[test]
fn message_custom_from_proto_only() {
  let msg = MessageCustomFromProtoOnlyProto::default();

  let proxy: MessageCustomFromProtoOnly = msg.into();

  assert_eq_pretty!(proxy.id.0, 1);
}

#[proto_message(proxied)]
#[proto(skip_checks(all))]
#[proto(into_proto = |_| MessageCustomIntoProtoOnlyProto { id: 2 })]
pub struct MessageCustomIntoProtoOnly {
  #[allow(unused)]
  #[proto(int32)]
  id: IntWrapper,
}

#[test]
fn message_custom_into_proto_only() {
  let proxy = MessageCustomIntoProtoOnly { id: IntWrapper(0) };

  let msg = proxy.into_message();

  assert_eq_pretty!(msg.id, 2);
}

impl Default for OneofIgnoredFieldDefaultConversionProto {
  fn default() -> Self {
    Self::B(1)
  }
}

#[proto_oneof(proxied)]
// Implicitly checks that proto derives are compiling
#[proto(derive(Copy))]
#[proto(skip_checks(all))]
pub enum OneofIgnoredFieldDefaultConversion {
  // This will use Default
  #[allow(unused)]
  #[proto(ignore)]
  A(String),
  #[proto(tag = 1)]
  B(i32),
  #[proto(tag = 2)]
  C(bool),
}

#[test]
fn oneof_ignored_field_default_conversion() {
  let proxy = OneofIgnoredFieldDefaultConversion::A("abc".to_string());

  let oneof = proxy.into_oneof();

  matches!(oneof, OneofIgnoredFieldDefaultConversionProto::B(1));
}

#[proto_oneof(proxied)]
#[proto(skip_checks(all))]
pub enum OneofIgnoredFieldCustomConversion {
  #[allow(unused)]
  #[proto(ignore)]
  #[proto(into_proto = |_| OneofIgnoredFieldCustomConversionProto::B(1))]
  A(String),
  #[proto(tag = 1)]
  B(i32),
  #[proto(tag = 2)]
  C(bool),
}

#[test]
fn oneof_ignored_field_custom_conversion() {
  let proxy = OneofIgnoredFieldCustomConversion::A("abc".to_string());

  let oneof = proxy.into_oneof();

  matches!(oneof, OneofIgnoredFieldCustomConversionProto::B(1));
}

fn custom_global_conv(
  input: OneofIgnoredGlobalCustomConversion,
) -> OneofIgnoredGlobalCustomConversionProto {
  match input {
    OneofIgnoredGlobalCustomConversion::A(_) | OneofIgnoredGlobalCustomConversion::B(_) => {
      OneofIgnoredGlobalCustomConversionProto::B(1)
    }
    OneofIgnoredGlobalCustomConversion::C(_) => OneofIgnoredGlobalCustomConversionProto::C(false),
  }
}

#[allow(unused)]
#[proto_oneof(proxied)]
#[proto(skip_checks(all))]
#[proto(into_proto = custom_global_conv)]
pub enum OneofIgnoredGlobalCustomConversion {
  #[allow(unused)]
  #[proto(ignore)]
  A(String),
  #[proto(tag = 1)]
  B(i32),
  #[proto(tag = 2)]
  C(bool),
}

#[test]
fn oneof_ignored_global_custom_conversion() {
  let proxy = OneofIgnoredGlobalCustomConversion::A("abc".to_string());

  let oneof = proxy.into_oneof();

  matches!(oneof, OneofIgnoredGlobalCustomConversionProto::B(1));
}

#[proto_message(proxied)]
#[proto(skip_checks(all))]
pub struct MessageIgnoredFieldDefault {
  #[proto(ignore)]
  #[allow(unused)]
  ignored: i32,
  other: u32,
}

#[test]
fn message_ignored_field_default() {
  let msg = MessageIgnoredFieldDefaultProto { other: 1 };

  let proxy: MessageIgnoredFieldDefault = msg.into();

  assert_eq_pretty!(proxy.ignored, 0);
}

#[proto_message(proxied)]
#[proto(skip_checks(all))]
pub struct MessageIgnoredFieldCustom {
  #[proto(ignore)]
  #[proto(from_proto = Default::default)]
  #[allow(unused)]
  ignored: i32,
  other: u32,
}

#[test]
fn message_ignored_field_custom() {
  let msg = MessageIgnoredFieldCustomProto { other: 1 };

  let proxy: MessageIgnoredFieldCustom = msg.into();

  assert_eq_pretty!(proxy.ignored, 0);
}

#[proto_message(proxied)]
#[proto(skip_checks(all))]
#[proto(from_proto = |v| MessageIgnoredFieldCustomGlobal { other: v.other, ignored: 1 })]
pub struct MessageIgnoredFieldCustomGlobal {
  #[proto(ignore)]
  #[allow(unused)]
  ignored: i32,
  other: u32,
}

#[test]
fn message_ignored_field_global() {
  let msg = MessageIgnoredFieldCustomGlobalProto { other: 1 };

  let proxy: MessageIgnoredFieldCustomGlobal = msg.into();

  assert_eq_pretty!(proxy.ignored, 1);
  assert_eq_pretty!(proxy.other, 1);
}

#[proto_message]
#[proto(skip_checks(all))]
pub struct MsgWithCustomProxy {
  pub id: i32,
}

pub struct CustomMsgProxy {
  pub id: i32,
}

impl From<CustomMsgProxy> for MsgWithCustomProxy {
  fn from(value: CustomMsgProxy) -> Self {
    Self { id: value.id }
  }
}

impl From<MsgWithCustomProxy> for CustomMsgProxy {
  fn from(value: MsgWithCustomProxy) -> Self {
    Self { id: value.id }
  }
}

#[proto_oneof]
#[proto(skip_checks(all))]
pub enum OneofWithCustomProxy {
  #[proto(tag = 1)]
  A(i32),
  #[proto(tag = 2)]
  B(i32),
}

impl Default for OneofWithCustomProxy {
  fn default() -> Self {
    Self::A(1)
  }
}

#[derive(Default)]
pub enum CustomOneofProxy {
  #[default]
  A,
  B,
}

impl From<CustomOneofProxy> for OneofWithCustomProxy {
  fn from(_: CustomOneofProxy) -> Self {
    Self::default()
  }
}

impl From<OneofWithCustomProxy> for CustomOneofProxy {
  fn from(_: OneofWithCustomProxy) -> Self {
    Self::default()
  }
}

#[proto_message(proxied)]
#[proto(skip_checks(all))]
pub struct CustomProxyTest {
  #[proto(message(MsgWithCustomProxy))]
  pub msg: Option<CustomMsgProxy>,
  #[proto(message(default, MsgWithCustomProxy))]
  pub with_default: CustomMsgProxy,
  #[proto(oneof(tags(1, 2), OneofWithCustomProxy))]
  pub oneof: Option<CustomOneofProxy>,
  #[proto(oneof(default, tags(3, 4), OneofWithCustomProxy))]
  pub oneof_with_default: CustomOneofProxy,
}

mod borderline_nonsensical {
  use super::*;

  // These are just stress tests for various scenarios to make sure that all conditions
  // that are technically allowed are compiling, although in practice most of these
  // would actually cause a stack overflow if they were to be actually converted

  #[proto_message(proxied)]
  #[proto(skip_checks(all))]
  pub struct ProxiedMessageWithDefault {
    #[proto(message(default, proxied))]
    recursive: Box<ProxiedMessageWithDefault>,
    #[proto(message(default))]
    direct: MessageWithDefault,
  }

  #[proto_message]
  #[proto(skip_checks(all))]
  pub struct MessageWithDefault {
    #[proto(message)]
    recursive: Option<Box<MessageWithDefault>>,
  }

  #[proto_message]
  #[proto(skip_checks(all))]
  pub struct WithDirectRecursiveOneof {
    #[proto(oneof(tags(1, 2)))]
    oneof: Option<DirectRecursiveOneof>,
  }

  #[proto_oneof]
  #[proto(skip_checks(all))]
  pub enum DirectRecursiveOneof {
    #[proto(tag = 1)]
    A(i32),
    #[proto(tag = 2, message)]
    B(Box<WithDirectRecursiveOneof>),
  }

  #[proto_message(proxied)]
  #[proto(skip_checks(all))]
  pub struct WithProxiedRecursiveDefaultOneof {
    #[proto(oneof(default, proxied, tags(1, 2)))]
    oneof: ProxiedRecursiveOneof,
  }

  #[proto_oneof(proxied)]
  #[proto(skip_checks(all))]
  pub enum ProxiedRecursiveOneof {
    #[proto(tag = 1)]
    A(i32),
    #[proto(tag = 2, message(proxied))]
    B(Box<WithProxiedRecursiveDefaultOneof>),
  }

  impl Default for ProxiedRecursiveOneofProto {
    fn default() -> Self {
      Self::A(1)
    }
  }

  // This should compile because a non-Option message should be allowed without `default`
  // if a custom conversion impl is provided
  #[proto_message(proxied)]
  #[proto(skip_checks(all))]
  pub struct DefaultMsgWithCustomImpl {
    #[proto(message(proxied))]
    #[proto(from_proto = |_| Box::new(DefaultMsgWithCustomImpl { recursive: Box::new(DefaultMsgWithCustomImplProto::default().into()), normal: DirectMsg::default() }))]
    #[proto(into_proto = |_| Some(Box::new(DefaultMsgWithCustomImplProto::default())))]
    recursive: Box<DefaultMsgWithCustomImpl>,
    #[proto(message)]
    #[proto(from_proto = |v| v.unwrap_or_default())]
    #[proto(into_proto = |_| Some(DirectMsg::default()))]
    normal: DirectMsg,
  }
}
