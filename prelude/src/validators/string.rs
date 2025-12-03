use bon::Builder;
use regex::Regex;
use string_validator_builder::{IsUnset, SetIn, SetNotIn, SetWellKnown, State};

use super::*;

impl_proto_type!(String, "string");

impl_into_option!(StringValidator);
impl_validator!(StringValidator, String);
impl_ignore!(StringValidatorBuilder);

impl<S: State> StringValidatorBuilder<S>
where
  S::In: IsUnset,
{
  /// Specifies that only the values in this list will be considered valid for this field.
  pub fn in_<T: Into<Arc<str>>, I: IntoIterator<Item = T>>(
    self,
    list: I,
  ) -> StringValidatorBuilder<SetIn<S>> {
    let list = create_string_list(list);
    self.in_internal(list)
  }
}

impl<S: State> StringValidatorBuilder<S>
where
  S::NotIn: IsUnset,
{
  /// Specifies that the values in this list will be considered NOT valid for this field.
  pub fn not_in<T: Into<Arc<str>>, I: IntoIterator<Item = T>>(
    self,
    list: I,
  ) -> StringValidatorBuilder<SetNotIn<S>> {
    let list = create_string_list(list);
    self.not_in_internal(list)
  }
}

#[derive(Clone, Debug, Builder)]
#[builder(derive(Clone))]
#[builder(on(Arc<str>, into))]
pub struct StringValidator {
  /// Specifies that the given string field must be of this exact length.
  pub len: Option<u64>,
  /// Specifies that the given string field must have a length that is equal to or higher than the given value.
  pub min_len: Option<u64>,
  /// Specifies that the given string field must have a length that is equal to or lower than the given value.
  pub max_len: Option<u64>,
  /// Specifies the exact byte length that this field's value must have in order to be considered valid.
  pub len_bytes: Option<u64>,
  /// Specifies the minimum byte length for this field's value to be considered valid.
  pub min_bytes: Option<u64>,
  /// Specifies the minimum byte length for this field's value to be considered valid.
  pub max_bytes: Option<u64>,
  /// Specifies a regex pattern that this field's value should match in order to be considered valid.
  pub pattern: Option<Regex>,
  /// Specifies the prefix that this field's value should contain in order to be considered valid.
  pub prefix: Option<Arc<str>>,
  /// Specifies the suffix that this field's value should contain in order to be considered valid.
  pub suffix: Option<Arc<str>>,
  /// Specifies a substring that this field's value should contain in order to be considered valid.
  pub contains: Option<Arc<str>>,
  /// Specifies a substring that this field's value must not contain in order to be considered valid.
  pub not_contains: Option<Arc<str>>,
  /// Specifies that only the values in this list will be considered valid for this field.
  #[builder(setters(vis = "", name = in_internal))]
  pub in_: Option<Arc<[Arc<str>]>>,
  /// Specifies that the values in this list will be considered NOT valid for this field.
  #[builder(setters(vis = "", name = not_in_internal))]
  pub not_in: Option<Arc<[Arc<str>]>>,
  #[builder(setters(vis = "", name = well_known))]
  pub well_known: Option<WellKnownStrings>,
  /// Specifies that only this specific value will be considered valid for this field.
  pub const_: Option<Arc<str>>,
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  #[builder(into)]
  pub cel: Option<Arc<[CelRule]>>,
  #[builder(with = || true)]
  /// Specifies that the field must be set in order to be valid.
  pub required: Option<bool>,
  #[builder(setters(vis = "", name = ignore))]
  pub ignore: Option<Ignore>,
}

impl From<StringValidator> for ProtoOption {
  fn from(validator: StringValidator) -> ProtoOption {
    let mut rules: OptionValueList = Vec::new();

    if let Some(const_val) = validator.const_ {
      rules.push((CONST_.clone(), OptionValue::String(const_val)));
    }

    if validator.len.is_none() {
      insert_option!(validator, rules, min_len);
      insert_option!(validator, rules, max_len);
    } else {
      insert_option!(validator, rules, len);
    }

    if validator.len_bytes.is_none() {
      insert_option!(validator, rules, min_bytes);
      insert_option!(validator, rules, max_bytes);
    } else {
      insert_option!(validator, rules, len_bytes);
    }

    if let Some(pattern) = validator.pattern {
      rules.push((
        PATTERN.clone(),
        OptionValue::String(pattern.as_str().into()),
      ))
    }

    insert_option!(validator, rules, prefix);
    insert_option!(validator, rules, suffix);
    insert_option!(validator, rules, contains);
    insert_option!(validator, rules, not_contains);
    insert_option!(validator, rules, in_);
    insert_option!(validator, rules, not_in);

    if let Some(v) = validator.well_known {
      v.to_option(&mut rules)
    }

    // This is the outer rule grouping, "(buf.validate.field)"
    let mut outer_rules: OptionValueList = vec![];

    outer_rules.push((STRING.clone(), OptionValue::Message(rules.into())));

    // These must be added on the outer grouping, as they are generic rules
    // It's (buf.validate.field).required, NOT (buf.validate.field).string.required
    insert_cel_rules!(validator, outer_rules);
    insert_option!(validator, outer_rules, required);
    insert_option!(validator, outer_rules, ignore);

    ProtoOption {
      name: BUF_VALIDATE_FIELD.clone(),
      value: OptionValue::Message(outer_rules.into()),
    }
  }
}

#[doc(hidden)]
#[derive(Clone, Debug, Copy)]
pub enum WellKnownStrings {
  Email,
  Hostname,
  Ip,
  Ipv4,
  Ipv6,
  Uri,
  UriRef,
  Address,
  Uuid,
  Tuuid,
  IpWithPrefixlen,
  Ipv4WithPrefixlen,
  Ipv6WithPrefixlen,
  IpPrefix,
  Ipv4Prefix,
  Ipv6Prefix,
  HostAndPort,
  HeaderNameLoose,
  HeaderNameStrict,
  HeaderValueLoose,
  HeaderValueStrict,
}

macro_rules! well_known_impl {
  ($name:ident, $doc:literal) => {
    paste::paste! {
      #[doc = $doc]
      pub fn [< $name:snake >](self) -> StringValidatorBuilder<SetWellKnown<S>>
        where
          S::WellKnown: IsUnset,
        {
          self.well_known(WellKnownStrings::$name)
        }
    }
  };
}

impl<S: State> StringValidatorBuilder<S> {
  well_known_impl!(
    Email,
    r###"
    `email` specifies that the field value must be a valid email address, for
    example "foo@example.com".
    Conforms to the definition for a valid email address from the [HTML standard](https://html.spec.whatwg.org/multipage/input.html#valid-e-mail-address).
    Note that this standard willfully deviates from [RFC 5322](https://datatracker.ietf.org/doc/html/rfc5322),
    which allows many unexpected forms of email addresses and will easily match
    a typographical error.
  "###
  );
  well_known_impl!(
    Hostname,
    r###"
     `hostname` specifies that the field value must be a valid hostname, for
     example "foo.example.com".
    
     A valid hostname follows the rules below:
     - The name consists of one or more labels, separated by a dot (".").
     - Each label can be 1 to 63 alphanumeric characters.
     - A label can contain hyphens ("-"), but must not start or end with a hyphen.
     - The right-most label must not be digits only.
     - The name can have a trailing dot—for example, "foo.example.com.".
     - The name can be 253 characters at most, excluding the optional trailing dot.
  "###
  );
  well_known_impl!(
    Ip,
    r###"
    `ip` specifies that the field value must be a valid IP (v4 or v6) address.

    IPv4 addresses are expected in the dotted decimal format—for example, "192.168.5.21".
    IPv6 addresses are expected in their text representation—for example, "::1",
    or "2001:0DB8:ABCD:0012::0".
    
    Both formats are well-defined in the internet standard [RFC 3986](https://datatracker.ietf.org/doc/html/rfc3986).
    Zone identifiers for IPv6 addresses (for example, "fe80::a%en1") are supported.
  "###
  );
  well_known_impl!(
    Ipv4,
    r###"
    `ipv4` specifies that the field value must be a valid IPv4 address—for
    example "192.168.5.21".
  "###
  );
  well_known_impl!(
    Ipv6,
    r###"
    `ipv6` specifies that the field value must be a valid IPv6 address—for
    example "::1", or "d7a:115c:a1e0:ab12:4843:cd96:626b:430b".
  "###
  );
  well_known_impl!(
    Uri,
    r###"
    `uri` specifies that the field value must be a valid URI, for example
    "https://example.com/foo/bar?baz=quux#frag".
    
    URI is defined in the internet standard [RFC 3986](https://datatracker.ietf.org/doc/html/rfc3986).
    Zone Identifiers in IPv6 address literals are supported ([RFC 6874](https://datatracker.ietf.org/doc/html/rfc6874)).
  "###
  );
  well_known_impl!(
    UriRef,
    r###"
    `uri_ref` specifies that the field value must be a valid URI Reference—either
    a URI such as "https://example.com/foo/bar?baz=quux#frag", or a Relative
    Reference such as "./foo/bar?query".

    URI, URI Reference, and Relative Reference are defined in the internet
    standard [RFC 3986](https://datatracker.ietf.org/doc/html/rfc3986). Zone
    Identifiers in IPv6 address literals are supported ([RFC 6874](https://datatracker.ietf.org/doc/html/rfc6874)).
  "###
  );
  well_known_impl!(
    Address,
    r###"
    `address` specifies that the field value must be either a valid hostname
    (for example, "example.com"), or a valid IP (v4 or v6) address (for example,
    "192.168.0.1", or "::1").
  "###
  );
  well_known_impl!(
    Uuid,
    r###"
    `uuid` specifies that the field value must be a valid UUID as defined by
    [RFC 4122](https://datatracker.ietf.org/doc/html/rfc4122#section-4.1.2).
  "###
  );
  well_known_impl!(
    Tuuid,
    r###"
    `tuuid` (trimmed UUID) specifies that the field value must be a valid UUID as
    defined by [RFC 4122](https://datatracker.ietf.org/doc/html/rfc4122#section-4.1.2) with all dashes
    omitted.
  "###
  );
  well_known_impl!(
    IpWithPrefixlen,
    r###"
    `ip_with_prefixlen` specifies that the field value must be a valid IP
    (v4 or v6) address with prefix length—for example, "192.168.5.21/16" or
    "2001:0DB8:ABCD:0012::F1/64".
  "###
  );
  well_known_impl!(
    Ipv4WithPrefixlen,
    r###"
    `ipv4_with_prefixlen` specifies that the field value must be a valid
    IPv4 address with prefix length—for example, "192.168.5.21/16".
  "###
  );
  well_known_impl!(
    Ipv6WithPrefixlen,
    r###"
    `ipv6_with_prefixlen` specifies that the field value must be a valid
    IPv6 address with prefix length—for example, "2001:0DB8:ABCD:0012::F1/64".
  "###
  );
  well_known_impl!(
    IpPrefix,
    r###"
    `ip_prefix` specifies that the field value must be a valid IP (v4 or v6)
    prefix—for example, "192.168.0.0/16" or "2001:0DB8:ABCD:0012::0/64".

    The prefix must have all zeros for the unmasked bits. For example,
    "2001:0DB8:ABCD:0012::0/64" designates the left-most 64 bits for the
    prefix, and the remaining 64 bits must be zero.
  "###
  );
  well_known_impl!(
    Ipv4Prefix,
    r###"
    `ipv4_prefix` specifies that the field value must be a valid IPv4
    prefix, for example "192.168.0.0/16".

    The prefix must have all zeros for the unmasked bits. For example,
    "192.168.0.0/16" designates the left-most 16 bits for the prefix,
    and the remaining 16 bits must be zero.
  "###
  );
  well_known_impl!(
    Ipv6Prefix,
    r###"
    `ipv6_prefix` specifies that the field value must be a valid IPv6 prefix—for
    example, "2001:0DB8:ABCD:0012::0/64".

    The prefix must have all zeros for the unmasked bits. For example,
    "2001:0DB8:ABCD:0012::0/64" designates the left-most 64 bits for the
    prefix, and the remaining 64 bits must be zero.
  "###
  );
  well_known_impl!(
    HostAndPort,
    r###"
    `host_and_port` specifies that the field value must be valid host/port
    pair—for example, "example.com:8080".
    
    The host can be one of:
    - An IPv4 address in dotted decimal format—for example, "192.168.5.21".
    - An IPv6 address enclosed in square brackets—for example, "[2001:0DB8:ABCD:0012::F1]".
    - A hostname—for example, "example.com".
    
    The port is separated by a colon. It must be non-empty, with a decimal number
    in the range of 0-65535, inclusive.
  "###
  );
  well_known_impl!(
    HeaderNameLoose,
    r###"
    Specifies that the value must be a valid HTTP header name. 

    All characters are considered valid except for `\r\n\0`. 
    Use `header_name_strict` for stricter enforcement."###
  );
  well_known_impl!(
    HeaderNameStrict,
    r###"Specifies that the value must be a valid HTTP header name, according to the [RFC specification](https://datatracker.ietf.org/doc/html/rfc7230#section-3)"###
  );
  well_known_impl!(
    HeaderValueLoose,
    r###"
    Specifies that the value must be a valid HTTP header value. 

    All characters are considered valid except for `\r\n\0`. 
    Use `header_value_strict` for stricter enforcement."###
  );
  well_known_impl!(
    HeaderValueStrict,
    r###"Specifies that the value must be a valid HTTP header value, according to the [RFC specification](https://datatracker.ietf.org/doc/html/rfc7230#section-3)"###
  );
}

impl WellKnownStrings {
  pub(crate) fn to_option(self, option_values: &mut OptionValueList) {
    let name = match self {
      WellKnownStrings::Email => EMAIL.clone(),
      WellKnownStrings::Hostname => HOSTNAME.clone(),
      WellKnownStrings::Ip => IP.clone(),
      WellKnownStrings::Ipv4 => IPV4.clone(),
      WellKnownStrings::Ipv6 => IPV6.clone(),
      WellKnownStrings::Uri => URI.clone(),
      WellKnownStrings::UriRef => URI_REF.clone(),
      WellKnownStrings::Address => ADDRESS.clone(),
      WellKnownStrings::Uuid => UUID.clone(),
      WellKnownStrings::Tuuid => TUUID.clone(),
      WellKnownStrings::IpWithPrefixlen => IP_WITH_PREFIXLEN.clone(),
      WellKnownStrings::Ipv4WithPrefixlen => IPV4_WITH_PREFIXLEN.clone(),
      WellKnownStrings::Ipv6WithPrefixlen => IPV6_WITH_PREFIXLEN.clone(),
      WellKnownStrings::IpPrefix => IP_PREFIX.clone(),
      WellKnownStrings::Ipv4Prefix => IPV4_PREFIX.clone(),
      WellKnownStrings::Ipv6Prefix => IPV6_PREFIX.clone(),
      WellKnownStrings::HostAndPort => HOST_AND_PORT.clone(),
      WellKnownStrings::HeaderNameLoose
      | WellKnownStrings::HeaderNameStrict
      | WellKnownStrings::HeaderValueLoose
      | WellKnownStrings::HeaderValueStrict => WELL_KNOWN_REGEX.clone(),
    };

    let value = match self {
      WellKnownStrings::HeaderNameLoose => {
        option_values.push((STRICT.clone(), OptionValue::Bool(false)));
        OptionValue::Enum(KNOWN_REGEX_HTTP_HEADER_NAME.clone())
      }
      WellKnownStrings::HeaderNameStrict => {
        option_values.push((STRICT.clone(), OptionValue::Bool(true)));
        OptionValue::Enum(KNOWN_REGEX_HTTP_HEADER_NAME.clone())
      }
      WellKnownStrings::HeaderValueLoose => {
        option_values.push((STRICT.clone(), OptionValue::Bool(false)));
        OptionValue::Enum(KNOWN_REGEX_HTTP_HEADER_VALUE.clone())
      }
      WellKnownStrings::HeaderValueStrict => {
        option_values.push((STRICT.clone(), OptionValue::Bool(true)));
        OptionValue::Enum(KNOWN_REGEX_HTTP_HEADER_VALUE.clone())
      }
      _ => OptionValue::Bool(true),
    };

    option_values.push((name, value));
  }
}
