pub mod builder;
use builder::state::State;
pub use builder::{StringValidatorBuilder, WellKnownStrings};

#[cfg(feature = "regex")]
use regex::Regex;

use super::*;

#[derive(Clone, Debug)]
pub struct StringValidator {
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  pub cel: Vec<CelProgram>,

  pub well_known: Option<WellKnownStrings>,

  pub ignore: Ignore,

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
  pub in_: Option<StaticLookup<&'static str>>,

  /// Specifies that the values in this list will be considered NOT valid for this field.
  pub not_in: Option<StaticLookup<&'static str>>,

  /// Specifies that only this specific value will be considered valid for this field.
  pub const_: Option<Arc<str>>,
}

pub fn test_validator() -> StringValidatorBuilder {
  StringValidator::builder()
}

impl StringValidator {
  #[must_use]
  pub fn from_descriptor(desc: StringRules) -> Self {
    Self {
      cel: vec![],
      well_known: None,
      ignore: Ignore::Unspecified,
      required: false,
      len: desc.len.map(|v| v as usize),
      min_len: desc.min_len.map(|v| v as usize),
      max_len: desc.max_len.map(|v| v as usize),
      len_bytes: desc.len_bytes.map(|v| v as usize),
      min_bytes: desc.min_bytes.map(|v| v as usize),
      max_bytes: desc.max_bytes.map(|v| v as usize),
      #[cfg(feature = "regex")]
      pattern: desc.pattern.map(|p| Regex::new(&p).unwrap()),
      prefix: desc.prefix.map(|s| s.into()),
      suffix: desc.suffix.map(|s| s.into()),
      contains: desc.contains.map(|s| s.into()),
      not_contains: desc.not_contains.map(|s| s.into()),
      in_: (!desc.r#in.is_empty()).then(|| {
        StaticLookup::new(desc.r#in.into_iter().map(|s| {
          let str: &'static str = Box::leak(s.into_boxed_str());
          str
        }))
      }),
      not_in: None,
      const_: desc.r#const.map(|s| s.into()),
    }
  }

  #[inline]
  const fn has_pattern(&self) -> bool {
    #[cfg(feature = "regex")]
    {
      self.pattern.is_some()
    }
    #[cfg(not(feature = "regex"))]
    {
      false
    }
  }
}

#[cfg(feature = "regex")]
pub type CachedRegex = LazyLock<Regex>;

impl_proto_type!(String, "string");

impl_validator!(StringValidator, String);

impl Validator<String> for StringValidator {
  type Target = String;
  type UniqueStore<'a>
    = RefHybridStore<'a, String>
  where
    Self: 'a;

  #[inline]
  fn make_unique_store<'a>(&self, cap: usize) -> Self::UniqueStore<'a> {
    RefHybridStore::default_with_capacity(cap)
  }

  impl_testing_methods!();

  fn check_consistency(&self) -> Result<(), Vec<ConsistencyError>> {
    let mut errors = Vec::new();

    macro_rules! check_prop_some {
      ($($id:ident),*) => {
        $(self.$id.is_some()) ||*
      };
    }

    if self.const_.is_some()
      && (!self.cel.is_empty()
        || check_prop_some!(
          in_,
          not_in,
          well_known,
          len,
          min_len,
          max_len,
          len_bytes,
          min_bytes,
          max_bytes,
          suffix,
          prefix,
          contains,
          not_contains
        )
        || self.has_pattern())
    {
      errors.push(ConsistencyError::ConstWithOtherRules);
    }

    #[cfg(feature = "cel")]
    if let Err(e) = self.check_cel_programs() {
      errors.extend(e.into_iter().map(ConsistencyError::from));
    }

    if let Some(contains) = self.contains.as_ref()
      && let Some(not_contains) = self.not_contains.as_ref()
      && contains == not_contains
    {
      errors.push(ConsistencyError::ContradictoryInput(
        "`contains` and `not_contains` have the same value".to_string(),
      ));
    }

    if let Err(e) = check_list_rules(self.in_.as_ref(), self.not_in.as_ref()) {
      errors.push(e.into());
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

  fn validate(&self, ctx: &mut ValidationCtx, val: Option<&Self::Target>) {
    handle_ignore_always!(&self.ignore);
    handle_ignore_if_zero_value!(&self.ignore, val.is_none_or(|v| v.is_default()));

    if let Some(val) = val {
      if let Some(const_val) = &self.const_ {
        if val != const_val.as_ref() {
          ctx.add_violation(
            &STRING_CONST_VIOLATION,
            &format!("must be equal to {const_val}",),
          );
        }

        // Using `const` implies no other rules
        return;
      }

      if let Some(len) = self.len
        && val.chars().count() != len
      {
        ctx.add_violation(
          &STRING_LEN_VIOLATION,
          &format!("must be exactly {len} characters long"),
        );
      }

      if let Some(min_len) = self.min_len
        && val.chars().count() < min_len
      {
        ctx.add_violation(
          &STRING_MIN_LEN_VIOLATION,
          &format!("must be at least {min_len} characters long"),
        );
      }

      if let Some(max_len) = self.max_len
        && val.chars().count() > max_len
      {
        ctx.add_violation(
          &STRING_MAX_LEN_VIOLATION,
          &format!("cannot be longer than {max_len} characters"),
        );
      }

      if let Some(len_bytes) = self.len_bytes
        && val.len() != len_bytes
      {
        ctx.add_violation(
          &STRING_LEN_BYTES_VIOLATION,
          &format!("must be exactly {len_bytes} bytes long"),
        );
      }

      if let Some(min_bytes) = self.min_bytes
        && val.len() < min_bytes
      {
        ctx.add_violation(
          &STRING_MIN_BYTES_VIOLATION,
          &format!("must be at least {min_bytes} bytes long"),
        );
      }

      if let Some(max_bytes) = self.max_bytes
        && val.len() > max_bytes
      {
        ctx.add_violation(
          &STRING_MAX_BYTES_VIOLATION,
          &format!("cannot be longer than {max_bytes} bytes"),
        );
      }

      #[cfg(feature = "regex")]
      if let Some(pattern) = &self.pattern
        && !pattern.is_match(val)
      {
        ctx.add_violation(
          &STRING_PATTERN_VIOLATION,
          &format!("must match the pattern `{pattern}`"),
        );
      }

      if let Some(prefix) = &self.prefix
        && !val.starts_with(prefix.as_ref())
      {
        ctx.add_violation(
          &STRING_PREFIX_VIOLATION,
          &format!("must start with {prefix}"),
        );
      }

      if let Some(suffix) = &self.suffix
        && !val.ends_with(suffix.as_ref())
      {
        ctx.add_violation(&STRING_SUFFIX_VIOLATION, &format!("must end with {suffix}"));
      }

      if let Some(substring) = &self.contains
        && !val.contains(substring.as_ref())
      {
        ctx.add_violation(
          &STRING_CONTAINS_VIOLATION,
          &format!("must contain {substring}"),
        );
      }

      if let Some(substring) = &self.not_contains
        && val.contains(substring.as_ref())
      {
        ctx.add_violation(
          &STRING_NOT_CONTAINS_VIOLATION,
          &format!("cannot contain {substring}"),
        );
      }

      if let Some(allowed_list) = &self.in_
        && !<&str>::is_in(&val.as_str(), &allowed_list.items)
      {
        let err = ["must be one of these values: ", &allowed_list.items_str].concat();

        ctx.add_violation(&STRING_IN_VIOLATION, &err);
      }

      if let Some(forbidden_list) = &self.not_in
        && <&str>::is_in(&val.as_str(), &forbidden_list.items)
      {
        let err = ["cannot be one of these values: ", &forbidden_list.items_str].concat();

        ctx.add_violation(&STRING_NOT_IN_VIOLATION, &err);
      }

      macro_rules! impl_well_known_check {
        ($check:expr, $violation:ident, $msg:literal) => {
          paste::paste! {
            if !$check(val.as_ref()) {
              ctx.add_violation(
                &[< STRING_ $violation _VIOLATION >],
                concat!("must be a valid ", $msg),
              );
            }
          }
        };
      }

      if let Some(well_known) = &self.well_known {
        match well_known {
          #[cfg(feature = "regex")]
          WellKnownStrings::Ulid => {
            impl_well_known_check!(is_valid_ulid, ULID, "ULID");
          }
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
              ctx.add_violation(
                &STRING_WELL_KNOWN_REGEX_VIOLATION,
                "must be a valid http header name",
              );
            }
          }
          #[cfg(feature = "regex")]
          WellKnownStrings::HeaderNameStrict => {
            if !is_valid_http_header_name(val.as_ref(), true) {
              ctx.add_violation(
                &STRING_WELL_KNOWN_REGEX_VIOLATION,
                "must be a valid http header name",
              );
            }
          }
          #[cfg(feature = "regex")]
          WellKnownStrings::HeaderValueLoose => {
            if !is_valid_http_header_value(val.as_ref(), false) {
              ctx.add_violation(
                &STRING_WELL_KNOWN_REGEX_VIOLATION,
                "must be a valid http header value",
              );
            }
          }
          #[cfg(feature = "regex")]
          WellKnownStrings::HeaderValueStrict => {
            if !is_valid_http_header_value(val.as_ref(), true) {
              ctx.add_violation(
                &STRING_WELL_KNOWN_REGEX_VIOLATION,
                "must be a valid http header value",
              );
            }
          }
        };
      }

      #[cfg(feature = "cel")]
      if !self.cel.is_empty() {
        let ctx = ProgramsExecutionCtx {
          programs: &self.cel,
          value: val.clone(),
          violations: ctx.violations,
          field_context: Some(&ctx.field_context),
          parent_elements: ctx.parent_elements,
        };

        ctx.execute_programs();
      }
    } else if self.required {
      ctx.add_required_violation();
    }
  }
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
      rules.push((
        IN_.clone(),
        OptionValue::new_list(allowed_list.items.iter()),
      ));
    }

    if let Some(forbidden_list) = &validator.not_in {
      rules.push((
        NOT_IN.clone(),
        OptionValue::new_list(forbidden_list.items.iter()),
      ));
    }

    if let Some(v) = validator.well_known {
      v.to_option(&mut rules)
    }

    // This is the outer rule grouping, "(buf.validate.field)"
    let mut outer_rules: OptionValueList = vec![];

    // (buf.validate.field).string .et_cetera...
    outer_rules.push((STRING.clone(), OptionValue::Message(rules.into())));

    // These must be added on the outer grouping, as they are generic rules
    // It's (buf.validate.field).required, NOT (buf.validate.field).string.required
    insert_cel_rules!(validator, outer_rules);
    insert_boolean_option!(validator, outer_rules, required);

    if !validator.ignore.is_default() {
      outer_rules.push((IGNORE.clone(), validator.ignore.into()))
    }

    Self {
      name: BUF_VALIDATE_FIELD.clone(),
      value: OptionValue::Message(outer_rules.into()),
    }
  }
}
