use crate::*;

pub struct TagAllocator<'a> {
  pub unavailable: &'a [Range<i32>],
  pub reserved_to_max: bool,
  pub next_tag: i32,
}

pub const PROTOBUF_MAX_TAG: i32 = 536_870_911;

impl<'a> TagAllocator<'a> {
  pub fn new(unavailable: &'a [Range<i32>]) -> Self {
    let reserved_to_max = unavailable
      .last()
      .is_some_and(|last| last.end > PROTOBUF_MAX_TAG + 1);

    Self {
      unavailable,
      next_tag: 1,
      reserved_to_max,
    }
  }

  #[track_caller]
  pub fn next_tag(&mut self) -> i32 {
    loop {
      let idx = self.unavailable.partition_point(|r| r.end <= self.next_tag);

      if let Some(range) = self.unavailable.get(idx)
        && range.contains(&self.next_tag) {
          self.next_tag = range.end;
          continue;
        }

      if self.reserved_to_max {
        panic!("Protobuf tag limit exceeded! Check if you have set the reserved numbers range to infinity");
      }

      let tag = self.next_tag;
      self.next_tag += 1;
      return tag;
    }
  }
}
