use std::cmp::Ordering;

use crate::*;

#[derive(Default, Clone, Debug)]
pub struct ReservedNumbers(pub Vec<Range<i32>>);

pub const PROTOBUF_MAX_TAG: i32 = 536_870_911;

fn is_reserved(id: i32, sorted_ranges: &[Range<i32>]) -> bool {
  let result = sorted_ranges.binary_search_by(|range| {
    if range.contains(&id) {
      Ordering::Equal
    } else if id < range.start {
      Ordering::Greater
    } else {
      Ordering::Less
    }
  });

  result.is_ok()
}

pub struct ManuallySetTag {
  pub tag: i32,
  pub field_span: Span,
}

pub fn sort_and_check_duplicate_tags(tags: &mut [ManuallySetTag]) -> syn::Result<()> {
  tags.sort_unstable_by_key(|mt| mt.tag);

  for i in 0..tags.len() {
    let ManuallySetTag { tag, field_span } = tags[i];

    if i > 0 && tag == tags[i - 1].tag {
      bail_with_span!(field_span, "Tag {tag} is used multiple times");
    }
  }

  Ok(())
}

pub fn build_unavailable_ranges2(
  reserved_numbers: &ReservedNumbers,
  manual_tags: &mut [ManuallySetTag],
) -> syn::Result<Vec<Range<i32>>> {
  manual_tags.sort_unstable_by_key(|mt| mt.tag);

  for i in 0..manual_tags.len() {
    let ManuallySetTag { tag, field_span } = manual_tags[i];

    if i > 0 && tag == manual_tags[i - 1].tag {
      bail_with_span!(field_span, "Tag {tag} is used multiple times");
    }

    if reserved_numbers.contains(tag) {
      bail_with_span!(field_span, "Tag {tag} conflicts with a reserved range");
    }
  }

  let mut reserved_iter = reserved_numbers.0.iter().cloned().peekable();

  let mut manual_iter = manual_tags
    .iter()
    .map(|mt| mt.tag..mt.tag + 1)
    .peekable();

  let mut merged = Vec::new();

  let mut get_next = || match (reserved_iter.peek(), manual_iter.peek()) {
    (Some(r), Some(m)) => {
      if r.start <= m.start {
        reserved_iter.next()
      } else {
        manual_iter.next()
      }
    }
    (Some(_), None) => reserved_iter.next(),
    (None, Some(_)) => manual_iter.next(),
    (None, None) => None,
  };

  let Some(mut current) = get_next() else {
    return Ok(vec![]);
  };

  while let Some(next) = get_next() {
    if next.start <= current.end {
      // Overlap or touching, coalesce
      current.end = std::cmp::max(current.end, next.end);
    } else {
      // Gap found
      merged.push(current);
      current = next;
    }
  }
  merged.push(current);

  Ok(merged)
}

impl ReservedNumbers {
  pub fn contains(&self, tag: i32) -> bool {
    is_reserved(tag, &self.0)
  }

  pub fn build_unavailable_ranges(self, manual_tags: &[i32]) -> Vec<Range<i32>> {
    if manual_tags.is_empty() {
      return self.0;
    }

    let mut ranges = self.0;

    for tag in manual_tags {
      ranges.push(*tag..(*tag + 1));
    }

    ranges.sort_by_key(|r| r.start);

    // Coalesce
    let mut merged: Vec<Range<i32>> = Vec::new();
    let mut current = ranges[0].clone();

    for next in ranges.into_iter().skip(1) {
      if next.start <= current.end {
        // Extend current to the max end
        current.end = std::cmp::max(current.end, next.end);
      } else {
        // Gap found, push current and start new
        merged.push(current);
        current = next;
      }
    }
    merged.push(current);

    merged
  }
}

impl ToTokens for ReservedNumbers {
  fn to_tokens(&self, tokens: &mut TokenStream2) {
    let mut agg_tokens = TokenStream2::new();

    for range in &self.0 {
      let start = range.start;
      let end = range.end;

      agg_tokens.extend(quote! {
        #start..#end,
      });
    }

    tokens.extend(agg_tokens);
  }
}

impl Parse for ReservedNumbers {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let mut ranges: Vec<Range<i32>> = Vec::new();

    let items = Punctuated::<Expr, Token![,]>::parse_terminated(input)?;

    for item in items {
      if let Expr::Range(range_expr) = &item {
        let start = if let Some(start_expr) = &range_expr.start {
          start_expr.as_int::<i32>()?
        } else {
          0
        };

        let end = if let Some(end_expr) = &range_expr.end {
          match &**end_expr {
            Expr::Lit(lit) => {
              if let Lit::Int(int) = &lit.lit
                && let Ok(num) = int.base10_parse()
              {
                num
              } else {
                bail!(end_expr, "Expected a number or `MAX`")
              }
            }
            Expr::Path(path) if path.path.is_ident("MAX") => PROTOBUF_MAX_TAG + 1,
            _ => bail!(end_expr, "Expected a number or `MAX`"),
          }
        } else {
          return Err(input.error(
            "Reserved ranges cannot be open. Use MAX to reserve up to the maximum protobuf range",
          ));
        };

        let final_end = if let RangeLimits::HalfOpen(_) = &range_expr.limits {
          end
        } else {
          end + 1
        };

        ranges.push(start..final_end);
      } else if let Expr::Lit(lit) = &item
        && let Lit::Int(lit_int) = &lit.lit
      {
        let num = lit_int.base10_parse::<i32>()?;

        ranges.push(num..num + 1);
      } else {
        return Err(error!(item, "Expected a range (e.g. `1..5`, `10..=15`)"));
      }
    }

    ranges.sort_by_key(|range| range.start);

    Ok(Self(ranges))
  }
}
