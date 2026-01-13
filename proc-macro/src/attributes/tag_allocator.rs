use crate::*;

pub struct TagAllocator<'a> {
  pub unavailable: &'a [Range<i32>],
  pub reserved_to_max: bool,
  pub current_range_idx: usize,
  pub next_tag: i32,
}

impl<'a> TagAllocator<'a> {
  pub fn new(unavailable: &'a [Range<i32>]) -> Self {
    let reserved_to_max = unavailable
      .last()
      .is_some_and(|last| last.end > PROTOBUF_MAX_TAG);

    Self {
      unavailable,
      next_tag: 1,
      reserved_to_max,
      current_range_idx: 0,
    }
  }

  pub fn next_tag(&mut self, span: Span) -> syn::Result<i32> {
    while self.current_range_idx < self.unavailable.len() {
      let range = &self.unavailable[self.current_range_idx];

      if self.next_tag < range.start {
        // Found a gap before the next reserved range
        let tag = self.next_tag;
        self.next_tag += 1;
        return Ok(tag);
      } else if self.next_tag < range.end {
        // Inside a reserved range, jump to the end
        self.next_tag = range.end;
        self.current_range_idx += 1;
        // Continue loop to check against the NEW next_tag
      } else {
        // The current reserved range is behind us, move to the next one
        self.current_range_idx += 1;
      }
    }

    if self.reserved_to_max {
      bail_with_span!(
        span,
        "Protobuf tag limit exceeded! Check if you have set the reserved numbers range to MAX",
      );
    }

    // If we ran out of reserved ranges, every tag is now available
    let tag = self.next_tag;
    self.next_tag += 1;
    Ok(tag)
  }
}
