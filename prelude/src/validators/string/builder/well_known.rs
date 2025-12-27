use super::*;

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
  Ulid,
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
          StringValidatorBuilder {
            _state: PhantomData,
            cel: self.cel,
            well_known: Some(WellKnownStrings::$name),
            ignore: self.ignore,
            required: self.required,
            len: self.len,
            min_len: self.min_len,
            max_len: self.max_len,
            len_bytes: self.len_bytes,
            min_bytes: self.min_bytes,
            max_bytes: self.max_bytes,
            #[cfg(feature = "regex")]
            pattern: self.pattern,
            prefix: self.prefix,
            suffix: self.suffix,
            contains: self.contains,
            not_contains: self.not_contains,
            in_: self.in_,
            not_in: self.not_in,
            const_: self.const_,
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
      Self::Ulid => ULID.clone(),
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
