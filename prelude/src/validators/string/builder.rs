#[doc(hidden)]
pub mod state;
use super::well_known_strings::WellKnownStrings;
use crate::validators::*;
pub(crate) use state::*;

#[derive(Clone, Debug)]
pub struct StringValidatorBuilder<S: State = Empty> {
  _state: PhantomData<S>,

  data: StringValidator,
}

impl ProtoValidation for String {
  type Target = str;
  type Stored = Self;
  type Validator = StringValidator;
  type Builder = StringValidatorBuilder;

  type UniqueStore<'a>
    = RefHybridStore<'a, str>
  where
    Self: 'a;
}
impl<S: State> ValidatorBuilderFor<String> for StringValidatorBuilder<S> {
  type Target = str;
  type Validator = StringValidator;
  #[inline]
  fn build_validator(self) -> StringValidator {
    self.build()
  }
}

impl<S: State> Default for StringValidatorBuilder<S> {
  #[inline]
  fn default() -> Self {
    Self {
      _state: PhantomData,
      data: StringValidator::default(),
    }
  }
}

impl StringValidator {
  #[must_use]
  #[inline]
  pub fn builder() -> StringValidatorBuilder {
    StringValidatorBuilder::default()
  }
}

impl<S: State> From<StringValidatorBuilder<S>> for ProtoOption {
  fn from(value: StringValidatorBuilder<S>) -> Self {
    value.build().into()
  }
}

#[allow(
  clippy::must_use_candidate,
  clippy::use_self,
  clippy::return_self_not_must_use
)]
impl<S: State> StringValidatorBuilder<S> {
  custom_error_messages_method!(String);

  #[inline]
  pub fn cel(mut self, program: CelProgram) -> StringValidatorBuilder<S> {
    self.data.cel.push(program);

    StringValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn ignore_always(mut self) -> StringValidatorBuilder<SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    self.data.ignore = Ignore::Always;

    StringValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn ignore_if_zero_value(mut self) -> StringValidatorBuilder<SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    self.data.ignore = Ignore::IfZeroValue;

    StringValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn required(mut self) -> StringValidatorBuilder<SetRequired<S>>
  where
    S::Required: IsUnset,
  {
    self.data.required = true;

    StringValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn len(mut self, val: usize) -> StringValidatorBuilder<SetLen<S>>
  where
    S::Len: IsUnset,
  {
    self.data.len = Some(val);

    StringValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn min_len(mut self, val: usize) -> StringValidatorBuilder<SetMinLen<S>>
  where
    S::MinLen: IsUnset,
  {
    self.data.min_len = Some(val);

    StringValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn max_len(mut self, val: usize) -> StringValidatorBuilder<SetMaxLen<S>>
  where
    S::MaxLen: IsUnset,
  {
    self.data.max_len = Some(val);

    StringValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn len_bytes(mut self, val: usize) -> StringValidatorBuilder<SetLenBytes<S>>
  where
    S::LenBytes: IsUnset,
  {
    self.data.len_bytes = Some(val);

    StringValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn min_bytes(mut self, val: usize) -> StringValidatorBuilder<SetMinBytes<S>>
  where
    S::MinBytes: IsUnset,
  {
    self.data.min_bytes = Some(val);

    StringValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn max_bytes(mut self, val: usize) -> StringValidatorBuilder<SetMaxBytes<S>>
  where
    S::MaxBytes: IsUnset,
  {
    self.data.max_bytes = Some(val);

    StringValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  #[cfg(feature = "regex")]
  #[track_caller]
  pub fn pattern(mut self, val: impl IntoRegex) -> StringValidatorBuilder<SetPattern<S>>
  where
    S::Pattern: IsUnset,
  {
    self.data.pattern = Some(val.into_regex());

    StringValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn prefix<T: Into<FixedStr>>(mut self, val: T) -> StringValidatorBuilder<SetPrefix<S>>
  where
    S::Prefix: IsUnset,
  {
    self.data.prefix = Some(val.into());

    StringValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn suffix<T: Into<FixedStr>>(mut self, val: T) -> StringValidatorBuilder<SetSuffix<S>>
  where
    S::Suffix: IsUnset,
  {
    self.data.suffix = Some(val.into());

    StringValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn contains<T: Into<FixedStr>>(mut self, val: T) -> StringValidatorBuilder<SetContains<S>>
  where
    S::Contains: IsUnset,
  {
    self.data.contains = Some(val.into());

    StringValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn not_contains<T: Into<FixedStr>>(
    mut self,
    val: T,
  ) -> StringValidatorBuilder<SetNotContains<S>>
  where
    S::NotContains: IsUnset,
  {
    self.data.not_contains = Some(val.into());

    StringValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn in_(mut self, val: impl IntoSortedList<FixedStr>) -> StringValidatorBuilder<SetIn<S>>
  where
    S::In: IsUnset,
  {
    self.data.in_ = Some(val.into_sorted_list());

    StringValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn not_in(mut self, val: impl IntoSortedList<FixedStr>) -> StringValidatorBuilder<SetNotIn<S>>
  where
    S::NotIn: IsUnset,
  {
    self.data.not_in = Some(val.into_sorted_list());

    StringValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn const_<T: Into<FixedStr>>(mut self, val: T) -> StringValidatorBuilder<SetConst<S>>
  where
    S::Const: IsUnset,
  {
    self.data.const_ = Some(val.into());

    StringValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn build(self) -> StringValidator {
    self.data
  }
}

macro_rules! well_known_impl {
  ($name:ident, $doc:literal) => {
    paste::paste! {
      #[doc = $doc]
      pub fn [< $name:snake >](mut self) -> StringValidatorBuilder<SetWellKnown<S>>
        where
          S::WellKnown: IsUnset,
        {
          self.data.well_known = Some(WellKnownStrings::$name);

          StringValidatorBuilder {
            _state: PhantomData,
            data: self.data,
          }
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
    Ulid,
    r"
    `ulid` specifies that the field value must be a valid ULID as defined by the
    [ULID specification](https://github.com/ulid/spec).
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
