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

impl WellKnownStrings {
  pub(crate) fn to_option(self) -> (Arc<str>, OptionValue, bool) {
    let mut is_strict = false;

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
      Self::HeaderNameLoose => OptionValue::Enum(KNOWN_REGEX_HTTP_HEADER_NAME.clone()),
      #[cfg(feature = "regex")]
      Self::HeaderNameStrict => {
        is_strict = true;
        OptionValue::Enum(KNOWN_REGEX_HTTP_HEADER_NAME.clone())
      }
      #[cfg(feature = "regex")]
      Self::HeaderValueLoose => OptionValue::Enum(KNOWN_REGEX_HTTP_HEADER_VALUE.clone()),
      #[cfg(feature = "regex")]
      Self::HeaderValueStrict => {
        is_strict = true;
        OptionValue::Enum(KNOWN_REGEX_HTTP_HEADER_VALUE.clone())
      }
      _ => OptionValue::Bool(true),
    };

    (name, value, is_strict)
  }
}

use std::{
  net::{IpAddr, Ipv6Addr},
  str::FromStr,
};

use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
#[cfg(feature = "regex")]
pub(crate) use regex_checks::*;

#[cfg(feature = "regex")]
mod regex_checks {
  use std::sync::LazyLock;

  use regex::Regex;

  static EMAIL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$").expect("Failed to create email regex")
  });

  pub(crate) fn is_valid_email(s: &str) -> bool {
    EMAIL_REGEX.is_match(s)
  }

  static HTTP_HEADER_NAME_STRICT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^:?[0-9a-zA-Z!#$%&'*+-.^_|~`]+$").unwrap());

  static HTTP_HEADER_NAME_LOOSE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[^\u0000\u000A\u000D]+$").unwrap());

  static HTTP_HEADER_VALUE_STRICT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[^\x00-\x08\x0A-\x1F\x7F]*$").unwrap());

  static HTTP_HEADER_VALUE_LOOSE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[^\u0000\u000A\u000D]*$").unwrap());

  #[must_use]
  pub(crate) fn is_valid_http_header_name(s: &str, strict: bool) -> bool {
    if s.is_empty() {
      return false;
    }

    let re = if strict {
      &HTTP_HEADER_NAME_STRICT_REGEX
    } else {
      &HTTP_HEADER_NAME_LOOSE_REGEX
    };

    re.is_match(s)
  }

  #[must_use]
  pub(crate) fn is_valid_http_header_value(s: &str, strict: bool) -> bool {
    if s.is_empty() {
      return false;
    }

    let re = if strict {
      &HTTP_HEADER_VALUE_STRICT_REGEX
    } else {
      &HTTP_HEADER_VALUE_LOOSE_REGEX
    };

    re.is_match(s)
  }

  pub(crate) fn is_valid_ulid(val: &str) -> bool {
    if val.is_empty() {
      return false;
    }

    static ULID_REGEX: LazyLock<Regex> =
      // Case insensitive (?i), strict first char check
      LazyLock::new(|| Regex::new(r"(?i)^[0-7][0-9A-HJKMNP-TV-Z]{25}$").unwrap());

    ULID_REGEX.is_match(val)
  }

  pub(crate) fn is_valid_uuid(s: &str) -> bool {
    static UUID_REGEX: LazyLock<Regex> = LazyLock::new(|| {
      Regex::new(r"^(?i)[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$").unwrap()
    });

    if s.is_empty() {
      return false;
    }

    UUID_REGEX.is_match(s)
  }

  pub(crate) fn is_valid_tuuid(s: &str) -> bool {
    static TUUID_REGEX: LazyLock<Regex> =
      LazyLock::new(|| Regex::new(r"^(?i)[0-9a-f]{32}$").unwrap());

    if s.is_empty() {
      return false;
    }

    TUUID_REGEX.is_match(s)
  }
}

#[must_use]
pub(crate) fn is_valid_uri(s: &str) -> bool {
  fluent_uri::Uri::parse(s).is_ok()
}

#[must_use]
pub(crate) fn is_valid_uri_ref(s: &str) -> bool {
  fluent_uri::UriRef::parse(s).is_ok()
}

#[must_use]
pub(crate) fn is_valid_ip_prefix(s: &str) -> bool {
  match IpNetwork::from_str(s) {
    Ok(network) => {
      //    Is the given IP address the same as the calculated network address?
      //    The .network() method returns the true prefix.
      //    The .ip() method returns the original IP from the string.
      network.ip() == network.network()
    }
    Err(_) => false,
  }
}

#[must_use]
pub(crate) fn is_valid_ipv4_prefix(s: &str) -> bool {
  match Ipv4Network::from_str(s) {
    Ok(network) => network.ip() == network.network(),
    Err(_) => false,
  }
}

#[must_use]
pub(crate) fn is_valid_ipv6_prefix(s: &str) -> bool {
  match Ipv6Network::from_str(s) {
    Ok(network) => network.ip() == network.network(),
    Err(_) => false,
  }
}

#[must_use]
pub(crate) fn is_valid_ip_with_prefixlen(s: &str) -> bool {
  IpNetwork::from_str(s).is_ok()
}

#[must_use]
pub(crate) fn is_valid_ipv4_with_prefixlen(s: &str) -> bool {
  Ipv4Network::from_str(s).is_ok()
}

#[must_use]
pub(crate) fn is_valid_ipv6_with_prefixlen(s: &str) -> bool {
  Ipv6Network::from_str(s).is_ok()
}

#[must_use]
pub(crate) fn is_valid_ip(s: &str) -> bool {
  s.parse::<IpAddr>().is_ok()
}

#[must_use]
pub(crate) fn is_valid_ipv4(s: &str) -> bool {
  s.parse::<IpAddr>().is_ok_and(|ip| ip.is_ipv4())
}

#[must_use]
pub(crate) fn is_valid_ipv6(s: &str) -> bool {
  s.parse::<IpAddr>().is_ok_and(|ip| ip.is_ipv6())
}

#[must_use]
pub(crate) fn is_valid_address(s: &str) -> bool {
  is_valid_hostname(s) || is_valid_ip(s)
}

#[must_use]
pub(crate) fn is_valid_hostname(hostname: &str) -> bool {
  let s = hostname.strip_suffix('.').unwrap_or(hostname);
  if s.len() > 253 {
    return false;
  }

  // Split the hostname into labels.
  let labels: Vec<&str> = s.split('.').collect();

  let last_label = match labels.last() {
    Some(label) => *label,
    None => return false, // Handles empty string case
  };

  // Iterate and validate each label.
  for label in labels {
    // Rule: Each label can be 1 to 63 characters.
    if label.is_empty() || label.len() > 63 {
      return false;
    }

    // Rule: A label can contain hyphens, but must not start or end with one.
    if label.starts_with('-') || label.ends_with('-') {
      return false;
    }

    // Rule: Each label can be alphanumeric characters or hyphens.
    if !label
      .chars()
      .all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
      return false;
    }
  }

  // Rule: The right-most label must not be digits only.
  if last_label.chars().all(|c| c.is_ascii_digit()) {
    return false;
  }

  true
}

#[must_use]
pub(crate) fn is_valid_port(port_str: &str) -> bool {
  // Port must not be empty.
  if port_str.is_empty() {
    return false;
  }
  // Cannot have leading zeros (unless it's just "0").
  if port_str.len() > 1 && port_str.starts_with('0') {
    return false;
  }
  // Try to parse as a u16, which covers the 0-65535 range automatically.
  port_str.parse::<u16>().is_ok()
}

#[must_use]
pub(crate) fn is_valid_host_and_port(s: &str) -> bool {
  if s.is_empty() {
    return false;
  }

  if let Some((host_part, port_part)) = s.rsplit_once(':') {
    // Is the host part a bracketed IPv6 address? e.g., `[::1]:8080`
    if let Some(ip_part) = host_part
      .strip_prefix('[')
      .and_then(|s| s.strip_suffix(']'))
    {
      return ip_part.parse::<Ipv6Addr>().is_ok() && is_valid_port(port_part);
    }

    // Otherwise, the host must be a regular hostname or an IP address.
    // `IpAddr` will handle both IPv4 and unbracketed IPv6.
    let is_host_valid = host_part.parse::<IpAddr>().is_ok() || is_valid_hostname(host_part);
    return is_host_valid && is_valid_port(port_part);
  }

  false
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn uris() {
    assert!(is_valid_uri(
      "https://middleeathtracker.com/hobbits?location=isengard"
    ));

    assert!(!is_valid_uri(
      "https://middleeathtracker.com/hobbits?location isengard"
    ));
  }

  #[test]
  fn name() {
    let ipv4_prefix = "192.168.0.0/16";
    let ipv4_with_prefixlen = "192.168.1.1/16";
    let ipv6_prefix = "2a01:c00::/24";
    let ipv6_with_prefixlen = "2a01:c23:7b6d:a900:1de7:5cbe:d8d2:f4a1/24";

    assert!(is_valid_ip_with_prefixlen(ipv4_with_prefixlen));
    assert!(is_valid_ip_with_prefixlen(ipv6_with_prefixlen));
    assert!(is_valid_ipv4_with_prefixlen(ipv4_with_prefixlen));
    assert!(!is_valid_ipv4_with_prefixlen(ipv6_with_prefixlen));
    assert!(is_valid_ipv6_with_prefixlen(ipv6_with_prefixlen));
    assert!(!is_valid_ipv6_with_prefixlen(ipv4_with_prefixlen));

    assert!(is_valid_ip_prefix(ipv4_prefix));
    assert!(is_valid_ip_prefix(ipv6_prefix));
    assert!(is_valid_ipv4_prefix(ipv4_prefix));
    assert!(!is_valid_ipv4_prefix(ipv6_prefix));
    assert!(!is_valid_ipv4_prefix(ipv4_with_prefixlen));
    assert!(is_valid_ipv6_prefix(ipv6_prefix));
    assert!(!is_valid_ipv6_prefix(ipv4_prefix));
    assert!(!is_valid_ipv6_prefix(ipv6_with_prefixlen));
  }

  #[test]
  fn network_identifiers() {
    let ipv4 = "192.168.1.1";
    let ipv6 = "2a01:c23:7b6d:a900:1de7:5cbe:d8d2:f4a1";

    assert!(is_valid_ip(ipv4));
    assert!(is_valid_ip(ipv6));
    assert!(is_valid_ipv4(ipv4));
    assert!(!is_valid_ipv4(ipv6));
    assert!(is_valid_ipv6(ipv6));
    assert!(!is_valid_ipv6(ipv4));

    assert!(is_valid_address("obiwan.force.com"));
    assert!(is_valid_address(ipv4));
    assert!(is_valid_address(ipv6));

    assert!(is_valid_host_and_port("obiwan.force:8080"));
    assert!(is_valid_host_and_port("192.168.1.120:3000"));
    assert!(is_valid_host_and_port("[2001:0DB8:ABCD:0012::F1]:3000"));

    assert!(!is_valid_host_and_port("obiwan.force"));
    assert!(!is_valid_host_and_port("192.168.1.120"));
    assert!(!is_valid_host_and_port("2001:0DB8:ABCD:0012::F1"));

    assert!(is_valid_hostname("obiwan.force.com"));
    assert!(!is_valid_hostname("-anakin.darkforce.com"));
    assert!(!is_valid_hostname("anakin.darkforce.com-"));
    assert!(!is_valid_hostname("anakin.darkforce.0"));
  }

  #[cfg(feature = "regex")]
  mod regex_tests {
    use super::*;

    #[test]
    fn identifiers() {
      use super::{is_valid_email, is_valid_tuuid, is_valid_uuid};

      assert!(is_valid_email("obiwan@force.com"));
      assert!(!is_valid_email("anakin@dark@force.com"));

      assert!(is_valid_uuid("d3b8f2d5-7e10-4c6e-8a1a-3b9c7d4f6e2c"));
      assert!(!is_valid_uuid("d3b8f2d57e104c6e8a1a3b9c7d4f6e2c"));

      assert!(is_valid_tuuid("d3b8f2d57e104c6e8a1a3b9c7d4f6e2c"));
      assert!(!is_valid_tuuid("d3b8f2d5-7e10-4c6e-8a1a-3b9c7d4f6e2c"))
    }

    #[test]
    fn headers() {
      use super::{is_valid_http_header_name, is_valid_http_header_value};

      assert!(is_valid_http_header_name("content-type", true));
      assert!(is_valid_http_header_name(":authority", true));

      assert!(!is_valid_http_header_name("content type", true));
      assert!(!is_valid_http_header_name("X-My@Header", true));
      assert!(!is_valid_http_header_name("X-Héader", true));
      assert!(!is_valid_http_header_name("", true));

      assert!(is_valid_http_header_name("X-My@Header", false));
      assert!(is_valid_http_header_name("X-Héader", false));

      assert!(!is_valid_http_header_name("Header\u{0000}WithNul", false));
      assert!(!is_valid_http_header_name("Header\nWithNewline", false));
      assert!(!is_valid_http_header_name("header\rwithcr", false));
      assert!(!is_valid_http_header_name("", false));

      assert!(is_valid_http_header_value(
        "application/json; charset=uft-8",
        true
      ));

      assert!(!is_valid_http_header_value(
        "value\u{0000}with\u{0000}nul",
        true
      ));
      assert!(!is_valid_http_header_value(
        "value\u{0007}with\u{0007}bell",
        true
      ));
      assert!(!is_valid_http_header_value(
        "value\u{000B}with\u{000B}vt",
        true
      ));
      assert!(!is_valid_http_header_value(
        "value\u{007F}with\u{007F}del",
        true
      ));

      assert!(!is_valid_http_header_value(
        "value\u{0000}with\u{0000}nul",
        false
      ));
      assert!(!is_valid_http_header_value("value\nwith\nnewline", false));
      assert!(!is_valid_http_header_value("value\rwith\rcr", false));
    }

    #[test]
    fn test_valid_ulids() {
      // A standard generated ULID
      assert!(is_valid_ulid("01AN4Z07BY79KA1307SR9X4MV3"));

      // The absolute minimum ULID (Time 0, Random 0)
      assert!(is_valid_ulid("00000000000000000000000000"));

      // The absolute maximum ULID (Time limit, Random max)
      // Note: Must start with 7, not Z
      assert!(is_valid_ulid("7ZZZZZZZZZZZZZZZZZZZZZZZZZ"));
    }

    #[test]
    fn test_case_insensitivity() {
      // Lowercase should work
      assert!(is_valid_ulid("01an4z07by79ka1307sr9x4mv3"));
      // Mixed case should work
      assert!(is_valid_ulid("01An4z07bY79kA1307sR9x4mV3"));
    }

    #[test]
    fn test_invalid_length() {
      // Empty
      assert!(!is_valid_ulid(""));

      // Too short (25 chars)
      assert!(!is_valid_ulid("01AN4Z07BY79KA1307SR9X4MV"));

      // Too long (27 chars)
      assert!(!is_valid_ulid("01AN4Z07BY79KA1307SR9X4MV33"));
    }

    #[test]
    fn test_excluded_characters() {
      // Crockford Base32 excludes I, L, O, U to avoid confusion

      // Contains I
      assert!(!is_valid_ulid("01AN4Z07BY79KA1307SR9X4MVI"));
      // Contains L
      assert!(!is_valid_ulid("01AN4Z07BY79KA1307SR9X4MVL"));
      // Contains O
      assert!(!is_valid_ulid("01AN4Z07BY79KA1307SR9X4MVO"));
      // Contains U
      assert!(!is_valid_ulid("01AN4Z07BY79KA1307SR9X4MVU"));
    }

    #[test]
    fn test_invalid_symbols() {
      // Hyphens are not allowed in canonical ULIDs
      assert!(!is_valid_ulid("01AN4Z07-Y79KA1307SR9X4MV3"));
      // Other symbols
      assert!(!is_valid_ulid("01AN4Z07BY79KA1307SR9X4MV@"));
    }

    #[test]
    fn test_timestamp_overflow() {
      // The first character maps to the top bits of the 48-bit timestamp.
      // It can only be 0-7.
      // 8 would imply a timestamp larger than 48 bits allow.

      // Starts with 8 (Invalid)
      assert!(!is_valid_ulid("80000000000000000000000000"));

      // Starts with 9 (Invalid)
      assert!(!is_valid_ulid("90000000000000000000000000"));

      // Starts with A (Invalid)
      assert!(!is_valid_ulid("A0000000000000000000000000"));

      // Starts with Z (Invalid)
      assert!(!is_valid_ulid("Z0000000000000000000000000"));
    }
  }
}
