use ::proto_types::protovalidate::KnownRegex;
use ::proto_types::protovalidate::string_rules::WellKnown;

use super::*;

impl RulesCtx {
  pub fn get_string_validator(&self) -> BuilderTokens {
    let span = self.field_span;
    let mut builder = BuilderTokens::new(span, quote_spanned! {span=> StringValidator::builder() });

    self.tokenize_ignore(&mut builder);
    self.tokenize_required(&mut builder);
    self.tokenize_cel_rules(&mut builder);

    if let Some(RulesType::String(rules)) = &self.rules.r#type {
      if let Some(val) = &rules.r#const {
        builder.extend(quote_spanned! {span=> .const_(#val) });
      }

      macro_rules! len_rule {
        ($name:ident) => {
          if let Some($name) = rules.$name {
            #[allow(clippy::cast_possible_truncation)]
            let $name = $name as usize;

            builder.extend(quote_spanned! {span=> .$name(#$name) });
          }
        };
      }

      macro_rules! str_rule {
        ($name:ident) => {
          if let Some($name) = &rules.$name {
            builder.extend(quote_spanned! {span=> .$name(#$name) });
          }
        };
      }

      len_rule!(len);
      len_rule!(min_len);
      len_rule!(max_len);
      len_rule!(len_bytes);
      len_rule!(min_bytes);
      len_rule!(max_bytes);

      str_rule!(pattern);
      str_rule!(prefix);
      str_rule!(suffix);
      str_rule!(contains);
      str_rule!(not_contains);

      if !rules.r#in.is_empty() {
        let list = &rules.r#in;
        builder.extend(quote_spanned! {span=> .in_([ #(#list),* ]) });
      }

      if !rules.not_in.is_empty() {
        let list = &rules.not_in;
        builder.extend(quote_spanned! {span=> .not_in([ #(#list),* ]) });
      }

      if let Some(well_known) = rules.well_known {
        macro_rules! match_well_known {
          ($($name:ident),*) => {
            paste::paste! {
              #[allow(unreachable_patterns)]
              match well_known {
                $(
                  WellKnown::$name(true) => {
                    builder.extend(quote_spanned! {span=> .[< $name:snake >]() });
                  }
                )*
                _ => {}
              }
            }
          };
        }

        if let WellKnown::WellKnownRegex(num) = well_known {
          let regex = KnownRegex::try_from(num).unwrap_or_default();
          let strict = rules.strict();

          let method_suffix = if strict {
            new_ident("strict")
          } else {
            new_ident("loose")
          };

          match regex {
            KnownRegex::HttpHeaderName => {
              let method_ident = format_ident!("header_name_{method_suffix}");
              builder.extend(quote_spanned! {span=> .#method_ident() });
            }
            KnownRegex::HttpHeaderValue => {
              let method_ident = format_ident!("header_value_{method_suffix}");
              builder.extend(quote_spanned! {span=> .#method_ident() });
            }
            KnownRegex::Unspecified => {}
          };
        } else {
          match_well_known!(
            Email,
            Ip,
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
            Ulid
          );
        }
      }
    }

    builder
  }
}
