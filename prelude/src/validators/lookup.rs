use proto_types::Duration;

use super::*;

pub trait ListFormatter: Sized {
  fn format_list(items: &[Self]) -> String;
}

macro_rules! impl_standard_format {
  ($($t:ty),*) => {
    $(
      impl ListFormatter for $t {
        fn format_list(items: &[Self]) -> String {
          format!("{:?}", items)
        }
      }
    )*
  }
}

impl_standard_format!(i32, i64, u32, u64, f32, f64);

impl<T: protocheck_core::ordered_float::FloatCore + Debug> ListFormatter for OrderedFloat<T> {
  fn format_list(items: &[Self]) -> String {
    format!("{:?}", items)
  }
}

impl ListFormatter for &[u8] {
  fn format_list(items: &[Self]) -> String {
    let total_bytes: usize = items.iter().map(|v| v.len()).sum();

    // Worst case: every byte becomes "\xNN" (4 chars)
    let data_est = total_bytes * 4;
    let quotes_len = items.len() * 2;
    let sep_len = (items.len() - 1) * 2;
    let brackets = 2;

    let capacity = data_est + quotes_len + sep_len + brackets;

    let mut acc = String::with_capacity(capacity);
    acc.push('[');

    for (i, item) in items.iter().enumerate() {
      if i > 0 {
        acc.push_str(", ");
      }

      acc.push('"');
      write!(&mut acc, "{}", item.escape_ascii()).unwrap();
      acc.push('"');
    }

    acc.push(']');

    acc.shrink_to_fit();
    acc
  }
}

impl ListFormatter for &str {
  fn format_list(items: &[Self]) -> String {
    let data_len: usize = items.iter().map(|s| s.len()).sum();

    let quotes_len = items.len() * 2;
    let sep_len = (items.len() - 1) * 2;
    let brackets = 2;

    let capacity = data_len + quotes_len + sep_len + brackets;

    let mut acc = String::with_capacity(capacity);
    acc.push('[');

    for (i, item) in items.iter().enumerate() {
      if i > 0 {
        acc.push_str(", ");
      }
      acc.push('"');
      acc.push_str(item);
      acc.push('"');
    }

    acc.push(']');

    acc.shrink_to_fit();
    acc
  }
}

impl ListFormatter for Duration {
  fn format_list(items: &[Self]) -> String {
    let est_cap = (items.len() * 54) + 2;

    let mut acc = String::with_capacity(est_cap);
    acc.push('[');

    for (i, item) in items.iter().enumerate() {
      if i > 0 {
        acc.push_str(", ");
      }

      acc.push('"');
      let _ = write!(&mut acc, "{item}");
      acc.push('"');
    }

    acc.push(']');
    acc
  }
}

#[derive(Debug)]
pub struct StaticLookup<T: Ord + ListFormatter> {
  pub items: SortedList<T>,
  pub items_str: String,
}

impl<T: Ord + ListFormatter> StaticLookup<T> {
  pub fn new<I>(iter: I) -> Self
  where
    I: IntoIterator<Item = T>,
  {
    let items = SortedList::new(iter);

    let items_str = T::format_list(&items);

    Self { items, items_str }
  }
}
