use std::ops::Deref;

use ordered_float::OrderedFloat;
use proto_types::Duration;

use super::*;

#[inline]
fn clamp_capacity_for_unique_items_collection<T>(requested_cap: usize) -> usize {
  // 128KB Budget
  const MAX_BYTES: usize = 128 * 1024;
  let item_size = std::mem::size_of::<T>();

  // For ZSTs, uniqueness checks would fail after one insertion anyway
  if item_size == 0 {
    return 1;
  }

  let max_items = MAX_BYTES / item_size;

  requested_cap.min(max_items)
}

pub trait UniqueStore<'a> {
  type Item: ?Sized;

  fn default_with_capacity(cap: usize) -> Self;
  fn insert(&mut self, item: &'a Self::Item) -> bool;
}

// Just for checking uniqueness for messages
#[doc(hidden)]
pub struct LinearRefStore<'a, T>
where
  T: 'a + ?Sized,
{
  seen: Vec<&'a T>,
}

impl<'a, T> UniqueStore<'a> for LinearRefStore<'a, T>
where
  T: 'a + PartialEq + ?Sized,
{
  type Item = T;

  #[inline]
  fn default_with_capacity(cap: usize) -> Self {
    let clamped_cap = clamp_capacity_for_unique_items_collection::<&T>(cap);

    Self {
      seen: Vec::with_capacity(clamped_cap),
    }
  }

  #[inline]
  fn insert(&mut self, item: &'a T) -> bool {
    if self.seen.contains(&item) {
      false
    } else {
      self.seen.push(item);
      true
    }
  }
}

#[doc(hidden)]
#[derive(Default)]
pub struct FloatEpsilonStore<T>
where
  T: FloatCore + FloatEq<Tol = T>,
{
  seen: Vec<OrderedFloat<T>>,
  abs_tol: T,
  rel_tol: T,
}

impl<T> FloatEpsilonStore<T>
where
  T: FloatCore + FloatEq<Tol = T>,
{
  pub(crate) fn new(cap: usize, abs: T, rel: T) -> Self {
    let clamped_cap = clamp_capacity_for_unique_items_collection::<T>(cap);

    Self {
      seen: Vec::with_capacity(clamped_cap),
      abs_tol: abs,
      rel_tol: rel,
    }
  }

  pub(crate) fn check_neighbors(&self, idx: usize, item: T) -> bool {
    // Idx at insertion point
    if let Some(above) = self.seen.get(idx)
      && float_eq!(above.0, item, abs <= self.abs_tol, r2nd <= self.rel_tol)
    {
      return true;
    }

    // Idx before insertion point
    if idx > 0
      && let Some(below) = self.seen.get(idx - 1)
      && float_eq!(below.0, item, abs <= self.abs_tol, r2nd <= self.rel_tol)
    {
      return true;
    }

    false
  }
}

impl<'a, T> UniqueStore<'a> for FloatEpsilonStore<T>
where
  T: FloatCore + FloatEq<Tol = T> + Default + 'a,
{
  type Item = T;

  #[inline]
  fn default_with_capacity(cap: usize) -> Self {
    let clamped_cap = clamp_capacity_for_unique_items_collection::<T>(cap);

    Self {
      seen: Vec::with_capacity(clamped_cap),
      abs_tol: Default::default(),
      rel_tol: Default::default(),
    }
  }

  #[inline]
  fn insert(&mut self, item: &Self::Item) -> bool {
    let wrapped = OrderedFloat(*item);

    match self.seen.binary_search(&wrapped) {
      // Exact bit-for-bit match found
      Ok(_) => false,

      // No exact match. 'idx' is the insertion point.
      Err(idx) => {
        let is_duplicate = self.check_neighbors(idx, *item);

        if is_duplicate {
          false
        } else {
          self.seen.insert(idx, wrapped);
          true
        }
      }
    }
  }
}

#[doc(hidden)]
pub struct UnsupportedStore<T> {
  _marker: PhantomData<T>,
}

impl<T> Default for UnsupportedStore<T> {
  #[inline]
  fn default() -> Self {
    Self {
      _marker: PhantomData,
    }
  }
}

impl<'a, T> UniqueStore<'a> for UnsupportedStore<T> {
  type Item = T;

  #[inline]
  fn default_with_capacity(_size: usize) -> Self {
    Self::default()
  }

  #[inline]
  fn insert(&mut self, _item: &'a Self::Item) -> bool {
    true
  }
}

#[doc(hidden)]
pub enum RefHybridStore<'a, T>
where
  T: 'a + ?Sized,
{
  Small(Vec<&'a T>),
  Large(HashSet<&'a T>),
}

impl<'a, T> UniqueStore<'a> for RefHybridStore<'a, T>
where
  T: 'a + Eq + Hash + Ord + ?Sized,
{
  type Item = T;

  #[inline]
  fn default_with_capacity(cap: usize) -> Self {
    let clamped_cap = clamp_capacity_for_unique_items_collection::<&T>(cap);

    if cap <= 32 {
      Self::Small(Vec::with_capacity(clamped_cap))
    } else {
      Self::Large(HashSet::with_capacity(clamped_cap))
    }
  }

  #[inline]
  fn insert(&mut self, item: &'a T) -> bool {
    match self {
      Self::Small(vec) => match vec.binary_search(&item) {
        Ok(_) => false,
        Err(idx) => {
          vec.insert(idx, item);
          true
        }
      },
      Self::Large(set) => set.insert(item),
    }
  }
}

#[doc(hidden)]
pub enum CopyHybridStore<T> {
  Small(Vec<T>),
  Large(HashSet<T>),
}

impl<'a, T> UniqueStore<'a> for CopyHybridStore<T>
where
  T: 'a + Copy + Eq + Hash + Ord,
{
  type Item = T;

  #[inline]
  fn default_with_capacity(cap: usize) -> Self {
    let clamped_cap = clamp_capacity_for_unique_items_collection::<T>(cap);

    if cap <= 32 {
      Self::Small(Vec::with_capacity(clamped_cap))
    } else {
      Self::Large(HashSet::with_capacity(clamped_cap))
    }
  }

  #[inline]
  fn insert(&mut self, item: &'a T) -> bool {
    match self {
      Self::Small(vec) => match vec.binary_search(item) {
        Ok(_) => false,
        Err(idx) => {
          vec.insert(idx, *item);
          true
        }
      },
      Self::Large(set) => set.insert(*item),
    }
  }
}

#[derive(Debug, Clone)]
pub struct SortedList<T: Ord> {
  pub(crate) items: Arc<[T]>,
}

impl<T> SortedList<T>
where
  T: Ord,
{
  pub fn new<I: IntoIterator<Item = T>>(iter: I) -> Self {
    let mut items: Vec<T> = iter.into_iter().collect();

    items.sort_unstable();

    Self {
      items: items.into(),
    }
  }

  #[must_use]
  pub fn as_slice(&self) -> &[T] {
    &self.items
  }

  pub fn contains<B>(&self, item: &B) -> bool
  where
    T: Borrow<B>,
    B: Ord + ?Sized,
  {
    self
      .items
      .binary_search_by(|probe| probe.borrow().cmp(item))
      .is_ok()
  }

  pub fn iter(&self) -> std::slice::Iter<'_, T> {
    self.into_iter()
  }

  #[allow(clippy::len_without_is_empty)]
  #[must_use]
  pub fn len(&self) -> usize {
    self.items.len()
  }
}

impl<T: Ord> Deref for SortedList<T> {
  type Target = [T];

  fn deref(&self) -> &Self::Target {
    &self.items
  }
}

impl<T: Ord> AsRef<[T]> for SortedList<T> {
  fn as_ref(&self) -> &[T] {
    &self.items
  }
}

impl<'a, T: Ord> IntoIterator for &'a SortedList<T> {
  type Item = &'a T;
  type IntoIter = std::slice::Iter<'a, T>;

  fn into_iter(self) -> Self::IntoIter {
    self.items.iter()
  }
}

pub trait ListFormatter: Sized {
  fn format_list(items: &[Self]) -> String;
}

macro_rules! impl_format_for_nums {
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

impl_format_for_nums!(i32, i64, u32, u64, f32, f64);

impl<T: ordered_float::FloatCore + Debug> ListFormatter for OrderedFloat<T> {
  fn format_list(items: &[Self]) -> String {
    format!("{items:?}")
  }
}

impl ListFormatter for ::bytes::Bytes {
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

impl ListFormatter for SharedStr {
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

#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct StaticLookup<T: Ord + ListFormatter> {
  pub items: SortedList<T>,
  pub items_str: Arc<str>,
}

impl<T: Ord + ListFormatter> StaticLookup<T> {
  pub fn new<I>(iter: I) -> Self
  where
    I: IntoIterator<Item = T>,
  {
    let items = SortedList::new(iter);

    let items_str = T::format_list(&items);

    Self {
      items,
      items_str: items_str.into(),
    }
  }
}
