use bon::Builder;
#[cfg(feature = "regex")]
use regex::Regex;
use string_validator_builder::{IsUnset, SetWellKnown, State};

use super::*;

#[cfg(feature = "regex")]
pub type CachedRegex = LazyLock<Regex>;

impl_proto_type!(String, "string");

impl_into_option!(StringValidator);
impl_validator!(StringValidator, String);
impl_ignore!(StringValidatorBuilder);
impl_cel_method!(StringValidatorBuilder);

impl Validator<String> for StringValidator {
  type Target = String;
  type UniqueStore<'a>
    = RefHybridStore<'a, String>
  where
    Self: 'a;

  fn make_unique_store<'a>(&self, cap: usize) -> Self::UniqueStore<'a> {
    RefHybridStore::default_with_capacity(cap)
  }

  impl_testing_methods!();

  #[cfg(feature = "testing")]
  fn check_consistency(&self) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if let Some(regex) = self.pattern {
      // This checks if a cached regex panics at formation or not
      let _ = regex.is_match("abc");
    }

    if let Some(contains) = self.contains.as_ref()
      && let Some(not_contains) = self.not_contains.as_ref()
      && contains == not_contains
    {
      errors.push("`contains` and `not_contains` have the same value".to_string());
    }

    if let Err(e) = check_list_rules(self.in_, self.not_in) {
      errors.push(e.to_string());
    }

    if let Err(e) = check_length_rules(
      Some(length_rule_value!("len", self.len)),
      length_rule_value!("min_len", self.min_len),
      length_rule_value!("max_len", self.max_len),
    ) {
      errors.push(e);
    }

    if let Err(e) = check_length_rules(
      Some(length_rule_value!("len_bytes", self.len_bytes)),
      length_rule_value!("min_bytes", self.min_bytes),
      length_rule_value!("max_bytes", self.max_bytes),
    ) {
      errors.push(e);
    }

    if errors.is_empty() {
      Ok(())
    } else {
      Err(errors)
    }
  }

  fn validate(
    &self,
    field_context: &FieldContext,
    parent_elements: &mut Vec<FieldPathElement>,
    val: Option<&Self::Target>,
  ) -> Result<(), Violations> {
    handle_ignore_always!(&self.ignore);
    handle_ignore_if_zero_value!(&self.ignore, val.is_none_or(|v| v.is_default()));

    let mut violations_agg = Violations::new();
    let violations = &mut violations_agg;

    if let Some(val) = val {
      if let Some(const_val) = &self.const_
        && val != const_val.as_ref()
      {
        violations.add(
          field_context,
          parent_elements,
          &STRING_CONST_VIOLATION,
          &format!("must be equal to {const_val}",),
        );
      }

      if let Some(len) = self.len
        && val.chars().count() != len
      {
        violations.add(
          field_context,
          parent_elements,
          &STRING_LEN_VIOLATION,
          &format!("must be exactly {len} characters long"),
        );
      }

      if let Some(min_len) = self.min_len
        && val.chars().count() < min_len
      {
        violations.add(
          field_context,
          parent_elements,
          &STRING_MIN_LEN_VIOLATION,
          &format!("must be at least {min_len} characters long"),
        );
      }

      if let Some(max_len) = self.max_len
        && val.chars().count() > max_len
      {
        violations.add(
          field_context,
          parent_elements,
          &STRING_MAX_LEN_VIOLATION,
          &format!("cannot be longer than {max_len} characters"),
        );
      }

      if let Some(len_bytes) = self.len_bytes
        && val.len() != len_bytes
      {
        violations.add(
          field_context,
          parent_elements,
          &STRING_LEN_BYTES_VIOLATION,
          &format!("must be exactly {len_bytes} bytes long"),
        );
      }

      if let Some(min_bytes) = self.min_bytes
        && val.len() < min_bytes
      {
        violations.add(
          field_context,
          parent_elements,
          &STRING_MIN_BYTES_VIOLATION,
          &format!("must be at least {min_bytes} bytes long"),
        );
      }

      if let Some(max_bytes) = self.max_bytes
        && val.len() > max_bytes
      {
        violations.add(
          field_context,
          parent_elements,
          &STRING_MAX_BYTES_VIOLATION,
          &format!("cannot be longer than {max_bytes} bytes"),
        );
      }

      #[cfg(feature = "regex")]
      if let Some(pattern) = &self.pattern
        && !pattern.is_match(val)
      {
        violations.add(
          field_context,
          parent_elements,
          &STRING_PATTERN_VIOLATION,
          &format!("must match the pattern `{pattern}`"),
        );
      }

      if let Some(prefix) = &self.prefix
        && !val.starts_with(prefix.as_ref())
      {
        violations.add(
          field_context,
          parent_elements,
          &STRING_PREFIX_VIOLATION,
          &format!("must start with {prefix}"),
        );
      }

      if let Some(suffix) = &self.suffix
        && !val.ends_with(suffix.as_ref())
      {
        violations.add(
          field_context,
          parent_elements,
          &STRING_SUFFIX_VIOLATION,
          &format!("must end with {suffix}"),
        );
      }

      if let Some(substring) = &self.contains
        && !val.contains(substring.as_ref())
      {
        violations.add(
          field_context,
          parent_elements,
          &STRING_CONTAINS_VIOLATION,
          &format!("must contain {substring}"),
        );
      }

      if let Some(substring) = &self.not_contains
        && val.contains(substring.as_ref())
      {
        violations.add(
          field_context,
          parent_elements,
          &STRING_NOT_CONTAINS_VIOLATION,
          &format!("cannot contain {substring}"),
        );
      }

      if let Some(allowed_list) = &self.in_
        && !<&str>::is_in(&val.as_str(), allowed_list)
      {
        violations.add(
          field_context,
          parent_elements,
          &STRING_IN_VIOLATION,
          &format!(
            "must be one of these values: {}",
            format_list(allowed_list.iter())
          ),
        );
      }

      if let Some(forbidden_list) = &self.not_in
        && <&str>::is_in(&val.as_str(), forbidden_list)
      {
        violations.add(
          field_context,
          parent_elements,
          &STRING_NOT_IN_VIOLATION,
          &format!(
            "cannot be one of these values: {}",
            format_list(forbidden_list.iter())
          ),
        );
      }

      macro_rules! impl_well_known_check {
        ($check:expr, $violation:ident, $msg:literal) => {
          paste::paste! {
            if !$check(val.as_ref()) {
              violations.add(
                field_context,
                parent_elements,
                &[< STRING_ $violation _VIOLATION >],
                concat!("must be a valid ", $msg),
              );
            }
          }
        };
      }

      if let Some(well_known) = &self.well_known {
        match well_known {
          WellKnownStrings::Ip => {
            impl_well_known_check!(is_valid_ip, IP, "ip address");
          }
          WellKnownStrings::Ipv4 => {
            impl_well_known_check!(is_valid_ipv4, IPV4, "ipv4 address");
          }
          WellKnownStrings::Ipv6 => {
            impl_well_known_check!(is_valid_ipv6, IPV6, "ipv6 address");
          }
          #[cfg(feature = "regex")]
          WellKnownStrings::Email => {
            impl_well_known_check!(is_valid_email, EMAIL, "email address");
          }
          WellKnownStrings::Hostname => {
            impl_well_known_check!(is_valid_hostname, HOSTNAME, "hostname");
          }
          WellKnownStrings::Uri => {
            impl_well_known_check!(is_valid_uri, URI, "uri");
          }
          WellKnownStrings::UriRef => {
            impl_well_known_check!(is_valid_uri_ref, URI_REF, "uri reference");
          }
          WellKnownStrings::Address => {
            impl_well_known_check!(is_valid_address, ADDRESS, "address");
          }
          #[cfg(feature = "regex")]
          WellKnownStrings::Uuid => {
            impl_well_known_check!(is_valid_uuid, UUID, "uuid");
          }
          #[cfg(feature = "regex")]
          WellKnownStrings::Tuuid => {
            impl_well_known_check!(is_valid_tuuid, TUUID, "trimmed uuid");
          }
          WellKnownStrings::IpWithPrefixlen => {
            impl_well_known_check!(
              is_valid_ip_with_prefixlen,
              IP_WITH_PREFIXLEN,
              "ip with prefix length"
            );
          }
          WellKnownStrings::Ipv4WithPrefixlen => {
            impl_well_known_check!(
              is_valid_ipv4_with_prefixlen,
              IPV4_WITH_PREFIXLEN,
              "ipv4 with prefix length"
            );
          }
          WellKnownStrings::Ipv6WithPrefixlen => {
            impl_well_known_check!(
              is_valid_ipv6_with_prefixlen,
              IPV6_WITH_PREFIXLEN,
              "ipv6 with prefix length"
            );
          }
          WellKnownStrings::IpPrefix => {
            impl_well_known_check!(is_valid_ip_prefix, IP_PREFIX, "ip prefix");
          }
          WellKnownStrings::Ipv4Prefix => {
            impl_well_known_check!(is_valid_ipv4_prefix, IPV4_PREFIX, "ipv4 prefix");
          }
          WellKnownStrings::Ipv6Prefix => {
            impl_well_known_check!(is_valid_ipv6_prefix, IPV6_PREFIX, "ipv6 prefix");
          }
          WellKnownStrings::HostAndPort => {
            impl_well_known_check!(is_valid_host_and_port, HOST_AND_PORT, "host and port");
          }
          #[cfg(feature = "regex")]
          WellKnownStrings::HeaderNameLoose => {
            if !is_valid_http_header_name(val.as_ref(), false) {
              violations.add(
                field_context,
                parent_elements,
                &STRING_WELL_KNOWN_REGEX_VIOLATION,
                "must be a valid http header name",
              );
            }
          }
          #[cfg(feature = "regex")]
          WellKnownStrings::HeaderNameStrict => {
            if !is_valid_http_header_name(val.as_ref(), true) {
              violations.add(
                field_context,
                parent_elements,
                &STRING_WELL_KNOWN_REGEX_VIOLATION,
                "must be a valid http header name",
              );
            }
          }
          #[cfg(feature = "regex")]
          WellKnownStrings::HeaderValueLoose => {
            if !is_valid_http_header_value(val.as_ref(), false) {
              violations.add(
                field_context,
                parent_elements,
                &STRING_WELL_KNOWN_REGEX_VIOLATION,
                "must be a valid http header value",
              );
            }
          }
          #[cfg(feature = "regex")]
          WellKnownStrings::HeaderValueStrict => {
            if !is_valid_http_header_value(val.as_ref(), true) {
              violations.add(
                field_context,
                parent_elements,
                &STRING_WELL_KNOWN_REGEX_VIOLATION,
                "must be a valid http header value",
              );
            }
          }
        };
      }

      if !self.cel.is_empty() {
        let ctx = ProgramsExecutionCtx {
          programs: &self.cel,
          value: val.clone(),
          violations,
          field_context: Some(field_context),
          parent_elements,
        };

        ctx.execute_programs();
      }
    } else if self.required {
      violations.add_required(field_context, parent_elements);
    }

    if violations.is_empty() {
      Ok(())
    } else {
      Err(violations_agg)
    }
  }
}

#[derive(Clone, Debug, Builder)]
#[builder(derive(Clone))]
#[builder(on(Arc<str>, into))]
pub struct StringValidator {
  #[builder(field)]
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  pub cel: Vec<&'static CelProgram>,

  #[builder(setters(vis = "", name = well_known))]
  pub well_known: Option<WellKnownStrings>,

  #[builder(setters(vis = "", name = ignore))]
  pub ignore: Option<Ignore>,

  #[builder(default, with = || true)]
  /// Specifies that the field must be set in order to be valid.
  pub required: bool,

  /// Specifies that the given string field must be of this exact length.
  pub len: Option<usize>,

  /// Specifies that the given string field must have a length that is equal to or higher than the given value.
  pub min_len: Option<usize>,

  /// Specifies that the given string field must have a length that is equal to or lower than the given value.
  pub max_len: Option<usize>,

  /// Specifies the exact byte length that this field's value must have in order to be considered valid.
  pub len_bytes: Option<usize>,

  /// Specifies the minimum byte length for this field's value to be considered valid.
  pub min_bytes: Option<usize>,

  /// Specifies the minimum byte length for this field's value to be considered valid.
  pub max_bytes: Option<usize>,

  #[cfg(feature = "regex")]
  /// Specifies a regex pattern that this field's value should match in order to be considered valid.
  pub pattern: Option<&'static Regex>,

  /// Specifies the prefix that this field's value should contain in order to be considered valid.
  pub prefix: Option<Arc<str>>,

  /// Specifies the suffix that this field's value should contain in order to be considered valid.
  pub suffix: Option<Arc<str>>,

  /// Specifies a substring that this field's value should contain in order to be considered valid.
  pub contains: Option<Arc<str>>,

  /// Specifies a substring that this field's value must not contain in order to be considered valid.
  pub not_contains: Option<Arc<str>>,

  /// Specifies that only the values in this list will be considered valid for this field.
  pub in_: Option<&'static SortedList<&'static str>>,

  /// Specifies that the values in this list will be considered NOT valid for this field.
  pub not_in: Option<&'static SortedList<&'static str>>,

  /// Specifies that only this specific value will be considered valid for this field.
  pub const_: Option<Arc<str>>,
}

impl From<StringValidator> for ProtoOption {
  fn from(validator: StringValidator) -> Self {
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

    #[cfg(feature = "regex")]
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

    if let Some(allowed_list) = &validator.in_ {
      rules.push((IN_.clone(), OptionValue::new_list(allowed_list.iter())));
    }

    if let Some(forbidden_list) = &validator.not_in {
      rules.push((NOT_IN.clone(), OptionValue::new_list(forbidden_list.iter())));
    }

    if let Some(v) = validator.well_known {
      v.to_option(&mut rules)
    }

    // This is the outer rule grouping, "(buf.validate.field)"
    let mut outer_rules: OptionValueList = vec![];

    outer_rules.push((STRING.clone(), OptionValue::Message(rules.into())));

    // These must be added on the outer grouping, as they are generic rules
    // It's (buf.validate.field).required, NOT (buf.validate.field).string.required
    insert_cel_rules!(validator, outer_rules);
    insert_boolean_option!(validator, outer_rules, required);
    insert_option!(validator, outer_rules, ignore);

    Self {
      name: BUF_VALIDATE_FIELD.clone(),
      value: OptionValue::Message(outer_rules.into()),
    }
  }
}

#[doc(hidden)]
#[derive(Clone, Debug, Copy)]
pub enum WellKnownStrings {
  #[cfg(feature = "regex")]
  Email,
  Hostname,
  Ip,
  Ipv4,
  Ipv6,
  Uri,
  UriRef,
  Address,
  #[cfg(feature = "regex")]
  Uuid,
  #[cfg(feature = "regex")]
  Tuuid,
  IpWithPrefixlen,
  Ipv4WithPrefixlen,
  Ipv6WithPrefixlen,
  IpPrefix,
  Ipv4Prefix,
  Ipv6Prefix,
  HostAndPort,
  #[cfg(feature = "regex")]
  HeaderNameLoose,
  #[cfg(feature = "regex")]
  HeaderNameStrict,
  #[cfg(feature = "regex")]
  HeaderValueLoose,
  #[cfg(feature = "regex")]
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
  #[cfg(feature = "regex")]
  well_known_impl!(
    Email,
    r#"
    `email` specifies that the field value must be a valid email address, for
    example "foo@example.com".
    Conforms to the definition for a valid email address from the [HTML standard](https://html.spec.whatwg.org/multipage/input.html#valid-e-mail-address).
    Note that this standard willfully deviates from [RFC 5322](https://datatracker.ietf.org/doc/html/rfc5322),
    which allows many unexpected forms of email addresses and will easily match
    a typographical error.
  "#
  );
  well_known_impl!(
    Hostname,
    r#"
     `hostname` specifies that the field value must be a valid hostname, for
     example "foo.example.com".
    
     A valid hostname follows the rules below:
     - The name consists of one or more labels, separated by a dot (".").
     - Each label can be 1 to 63 alphanumeric characters.
     - A label can contain hyphens ("-"), but must not start or end with a hyphen.
     - The right-most label must not be digits only.
     - The name can have a trailing dot—for example, "foo.example.com.".
     - The name can be 253 characters at most, excluding the optional trailing dot.
  "#
  );
  well_known_impl!(
    Ip,
    r#"
    `ip` specifies that the field value must be a valid IP (v4 or v6) address.

    IPv4 addresses are expected in the dotted decimal format—for example, "192.168.5.21".
    IPv6 addresses are expected in their text representation—for example, "::1",
    or "2001:0DB8:ABCD:0012::0".
    
    Both formats are well-defined in the internet standard [RFC 3986](https://datatracker.ietf.org/doc/html/rfc3986).
    Zone identifiers for IPv6 addresses (for example, "fe80::a%en1") are supported.
  "#
  );
  well_known_impl!(
    Ipv4,
    r#"
    `ipv4` specifies that the field value must be a valid IPv4 address—for
    example "192.168.5.21".
  "#
  );
  well_known_impl!(
    Ipv6,
    r#"
    `ipv6` specifies that the field value must be a valid IPv6 address—for
    example "::1", or "d7a:115c:a1e0:ab12:4843:cd96:626b:430b".
  "#
  );
  well_known_impl!(
    Uri,
    r#"
    `uri` specifies that the field value must be a valid URI, for example
    "https://example.com/foo/bar?baz=quux#frag".
    
    URI is defined in the internet standard [RFC 3986](https://datatracker.ietf.org/doc/html/rfc3986).
    Zone Identifiers in IPv6 address literals are supported ([RFC 6874](https://datatracker.ietf.org/doc/html/rfc6874)).
  "#
  );
  well_known_impl!(
    UriRef,
    r#"
    `uri_ref` specifies that the field value must be a valid URI Reference—either
    a URI such as "https://example.com/foo/bar?baz=quux#frag", or a Relative
    Reference such as "./foo/bar?query".

    URI, URI Reference, and Relative Reference are defined in the internet
    standard [RFC 3986](https://datatracker.ietf.org/doc/html/rfc3986). Zone
    Identifiers in IPv6 address literals are supported ([RFC 6874](https://datatracker.ietf.org/doc/html/rfc6874)).
  "#
  );
  well_known_impl!(
    Address,
    r#"
    `address` specifies that the field value must be either a valid hostname
    (for example, "example.com"), or a valid IP (v4 or v6) address (for example,
    "192.168.0.1", or "::1").
  "#
  );
  #[cfg(feature = "regex")]
  well_known_impl!(
    Uuid,
    r"
    `uuid` specifies that the field value must be a valid UUID as defined by
    [RFC 4122](https://datatracker.ietf.org/doc/html/rfc4122#section-4.1.2).
  "
  );
  #[cfg(feature = "regex")]
  well_known_impl!(
    Tuuid,
    r"
    `tuuid` (trimmed UUID) specifies that the field value must be a valid UUID as
    defined by [RFC 4122](https://datatracker.ietf.org/doc/html/rfc4122#section-4.1.2) with all dashes
    omitted.
  "
  );
  well_known_impl!(
    IpWithPrefixlen,
    r#"
    `ip_with_prefixlen` specifies that the field value must be a valid IP
    (v4 or v6) address with prefix length—for example, "192.168.5.21/16" or
    "2001:0DB8:ABCD:0012::F1/64".
  "#
  );
  well_known_impl!(
    Ipv4WithPrefixlen,
    r#"
    `ipv4_with_prefixlen` specifies that the field value must be a valid
    IPv4 address with prefix length—for example, "192.168.5.21/16".
  "#
  );
  well_known_impl!(
    Ipv6WithPrefixlen,
    r#"
    `ipv6_with_prefixlen` specifies that the field value must be a valid
    IPv6 address with prefix length—for example, "2001:0DB8:ABCD:0012::F1/64".
  "#
  );
  well_known_impl!(
    IpPrefix,
    r#"
    `ip_prefix` specifies that the field value must be a valid IP (v4 or v6)
    prefix—for example, "192.168.0.0/16" or "2001:0DB8:ABCD:0012::0/64".

    The prefix must have all zeros for the unmasked bits. For example,
    "2001:0DB8:ABCD:0012::0/64" designates the left-most 64 bits for the
    prefix, and the remaining 64 bits must be zero.
  "#
  );
  well_known_impl!(
    Ipv4Prefix,
    r#"
    `ipv4_prefix` specifies that the field value must be a valid IPv4
    prefix, for example "192.168.0.0/16".

    The prefix must have all zeros for the unmasked bits. For example,
    "192.168.0.0/16" designates the left-most 16 bits for the prefix,
    and the remaining 16 bits must be zero.
  "#
  );
  well_known_impl!(
    Ipv6Prefix,
    r#"
    `ipv6_prefix` specifies that the field value must be a valid IPv6 prefix—for
    example, "2001:0DB8:ABCD:0012::0/64".

    The prefix must have all zeros for the unmasked bits. For example,
    "2001:0DB8:ABCD:0012::0/64" designates the left-most 64 bits for the
    prefix, and the remaining 64 bits must be zero.
  "#
  );
  well_known_impl!(
    HostAndPort,
    r#"
    `host_and_port` specifies that the field value must be valid host/port
    pair—for example, "example.com:8080".
    
    The host can be one of:
    - An IPv4 address in dotted decimal format—for example, "192.168.5.21".
    - An IPv6 address enclosed in square brackets—for example, "[2001:0DB8:ABCD:0012::F1]".
    - A hostname—for example, "example.com".
    
    The port is separated by a colon. It must be non-empty, with a decimal number
    in the range of 0-65535, inclusive.
  "#
  );
  #[cfg(feature = "regex")]
  well_known_impl!(
    HeaderNameLoose,
    r"
    Specifies that the value must be a valid HTTP header name. 

    All characters are considered valid except for `\r\n\0`. 
    Use `header_name_strict` for stricter enforcement."
  );
  #[cfg(feature = "regex")]
  well_known_impl!(
    HeaderNameStrict,
    r"Specifies that the value must be a valid HTTP header name, according to the [RFC specification](https://datatracker.ietf.org/doc/html/rfc7230#section-3)"
  );
  #[cfg(feature = "regex")]
  well_known_impl!(
    HeaderValueLoose,
    r"
    Specifies that the value must be a valid HTTP header value. 

    All characters are considered valid except for `\r\n\0`. 
    Use `header_value_strict` for stricter enforcement."
  );
  #[cfg(feature = "regex")]
  well_known_impl!(
    HeaderValueStrict,
    r"Specifies that the value must be a valid HTTP header value, according to the [RFC specification](https://datatracker.ietf.org/doc/html/rfc7230#section-3)"
  );
}

impl WellKnownStrings {
  pub(crate) fn to_option(self, option_values: &mut OptionValueList) {
    let name = match self {
      #[cfg(feature = "regex")]
      Self::Email => EMAIL.clone(),
      Self::Hostname => HOSTNAME.clone(),
      Self::Ip => IP.clone(),
      Self::Ipv4 => IPV4.clone(),
      Self::Ipv6 => IPV6.clone(),
      Self::Uri => URI.clone(),
      Self::UriRef => URI_REF.clone(),
      Self::Address => ADDRESS.clone(),
      #[cfg(feature = "regex")]
      Self::Uuid => UUID.clone(),
      #[cfg(feature = "regex")]
      Self::Tuuid => TUUID.clone(),
      Self::IpWithPrefixlen => IP_WITH_PREFIXLEN.clone(),
      Self::Ipv4WithPrefixlen => IPV4_WITH_PREFIXLEN.clone(),
      Self::Ipv6WithPrefixlen => IPV6_WITH_PREFIXLEN.clone(),
      Self::IpPrefix => IP_PREFIX.clone(),
      Self::Ipv4Prefix => IPV4_PREFIX.clone(),
      Self::Ipv6Prefix => IPV6_PREFIX.clone(),
      Self::HostAndPort => HOST_AND_PORT.clone(),
      #[cfg(feature = "regex")]
      Self::HeaderNameLoose
      | Self::HeaderNameStrict
      | Self::HeaderValueLoose
      | Self::HeaderValueStrict => WELL_KNOWN_REGEX.clone(),
    };

    let value = match self {
      #[cfg(feature = "regex")]
      Self::HeaderNameLoose => {
        option_values.push((STRICT.clone(), OptionValue::Bool(false)));
        OptionValue::Enum(KNOWN_REGEX_HTTP_HEADER_NAME.clone())
      }
      #[cfg(feature = "regex")]
      Self::HeaderNameStrict => {
        option_values.push((STRICT.clone(), OptionValue::Bool(true)));
        OptionValue::Enum(KNOWN_REGEX_HTTP_HEADER_NAME.clone())
      }
      #[cfg(feature = "regex")]
      Self::HeaderValueLoose => {
        option_values.push((STRICT.clone(), OptionValue::Bool(false)));
        OptionValue::Enum(KNOWN_REGEX_HTTP_HEADER_VALUE.clone())
      }
      #[cfg(feature = "regex")]
      Self::HeaderValueStrict => {
        option_values.push((STRICT.clone(), OptionValue::Bool(true)));
        OptionValue::Enum(KNOWN_REGEX_HTTP_HEADER_VALUE.clone())
      }
      _ => OptionValue::Bool(true),
    };

    option_values.push((name, value));
  }
}
