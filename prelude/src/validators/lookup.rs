use ordered_float::OrderedFloat;
use proto_types::Duration;

use super::*;

#[inline]
fn clamp_capacity_for_unique_items_collection<T>(requested_cap: usize) -> usize {
  // 128KB Budget
  const MAX_BYTES: usize = 128 * 1024;
  let item_size = core::mem::size_of::<T>();

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
pub struct UnsupportedStore<T: ?Sized> {
  _marker: PhantomData<T>,
}

#[allow(clippy::new_without_default, clippy::must_use_candidate)]
impl<T: ?Sized> UnsupportedStore<T> {
  #[inline(never)]
  #[cold]
  pub const fn new() -> Self {
    Self {
      _marker: PhantomData,
    }
  }
}

impl<'a, T: ?Sized> UniqueStore<'a> for UnsupportedStore<T> {
  type Item = T;

  #[inline(never)]
  #[cold]
  fn default_with_capacity(_size: usize) -> Self {
    Self::new()
  }

  #[inline(never)]
  #[cold]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SortedList<T: Ord> {
  pub(crate) items: Arc<[T]>,
}

pub trait IntoSortedList<T: Ord> {
  fn into_sorted_list(self) -> SortedList<T>;
}

impl<T: Ord> IntoSortedList<T> for SortedList<T> {
  #[allow(clippy::use_self)]
  #[inline]
  fn into_sorted_list(self) -> SortedList<T> {
    self
  }
}

impl<T: Ord + Clone> IntoSortedList<T> for &SortedList<T> {
  #[inline]
  fn into_sorted_list(self) -> SortedList<T> {
    self.clone()
  }
}

impl<T: Ord> IntoSortedList<T> for Vec<T> {
  fn into_sorted_list(self) -> SortedList<T> {
    SortedList::new(self)
  }
}

impl<T: Ord, const N: usize> IntoSortedList<T> for [T; N] {
  fn into_sorted_list(self) -> SortedList<T> {
    SortedList::new(self)
  }
}

impl<T: Ord + Copy> IntoSortedList<T> for &[T] {
  fn into_sorted_list(self) -> SortedList<T> {
    SortedList::new(self.iter().copied())
  }
}

impl<const N: usize> IntoSortedList<Bytes> for Vec<&'static [u8; N]> {
  fn into_sorted_list(self) -> SortedList<Bytes> {
    SortedList::new(self.into_iter().map(|b| Bytes::from_static(b)))
  }
}

impl IntoSortedList<Bytes> for &[&'static [u8]] {
  fn into_sorted_list(self) -> SortedList<Bytes> {
    SortedList::new(self.iter().map(|b| Bytes::from_static(b)))
  }
}

impl<const B: usize, const L: usize> IntoSortedList<Bytes> for [&'static [u8; B]; L] {
  fn into_sorted_list(self) -> SortedList<Bytes> {
    SortedList::new(self.into_iter().map(|b| Bytes::from_static(b)))
  }
}

macro_rules! impl_sorted_string_list {
  ($($typ:ty),*) => {
    $(
      impl IntoSortedList<FixedStr> for Vec<$typ> {
        fn into_sorted_list(self) -> SortedList<FixedStr> {
          let iter = self.into_iter().map(Into::into);
          SortedList::new(iter)
        }
      }

      impl<const N: usize> IntoSortedList<FixedStr> for [$typ; N] {
        fn into_sorted_list(self) -> SortedList<FixedStr> {
          let iter = self.into_iter().map(Into::into);
          SortedList::new(iter)
        }
      }
    )*
  };
}

impl_sorted_string_list!(String, Box<str>, &'static str, Arc<str>);

impl IntoSortedList<FixedStr> for &[&'static str] {
  fn into_sorted_list(self) -> SortedList<FixedStr> {
    let iter = self.iter().copied().map(Into::into);
    SortedList::new(iter)
  }
}

impl IntoSortedList<FixedStr> for &[Arc<str>] {
  fn into_sorted_list(self) -> SortedList<FixedStr> {
    let iter = self.iter().cloned().map(Into::into);
    SortedList::new(iter)
  }
}

macro_rules! impl_sorted_float_list {
  ($($typ:ty),*) => {
    $(
      impl IntoSortedList<OrderedFloat<$typ>> for Vec<$typ> {
        fn into_sorted_list(self) -> SortedList<OrderedFloat<$typ>> {
          let iter = self.into_iter().map(OrderedFloat);
          SortedList::new(iter)
        }
      }

      impl IntoSortedList<OrderedFloat<$typ>> for &[$typ] {
        fn into_sorted_list(self) -> SortedList<OrderedFloat<$typ>> {
          let iter = self.iter().copied().map(OrderedFloat);
          SortedList::new(iter)
        }
      }

      impl<const N: usize> IntoSortedList<OrderedFloat<$typ>> for [$typ; N] {
        fn into_sorted_list(self) -> SortedList<OrderedFloat<$typ>> {
          let iter = self.into_iter().map(OrderedFloat);
          SortedList::new(iter)
        }
      }
    )*
  };
}

impl_sorted_float_list!(f32, f64);

impl<T: Ord> FromIterator<T> for SortedList<T> {
  fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
    Self::new(iter)
  }
}

impl<T> SortedList<T>
where
  T: Ord,
{
  pub fn new<I: IntoIterator<Item = T>>(iter: I) -> Self {
    let mut items: Vec<T> = iter.into_iter().collect();

    items.sort_unstable();

    Self {
      items: items.into_boxed_slice().into(),
    }
  }

  #[must_use]
  #[inline]
  pub fn as_slice(&self) -> &[T] {
    &self.items
  }

  #[inline]
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

  #[inline]
  pub fn iter(&self) -> core::slice::Iter<'_, T> {
    self.into_iter()
  }

  #[allow(clippy::len_without_is_empty)]
  #[must_use]
  #[inline]
  pub fn len(&self) -> usize {
    self.items.len()
  }
}

impl<T: Ord> Deref for SortedList<T> {
  type Target = [T];

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.items
  }
}

impl<T: Ord> AsRef<[T]> for SortedList<T> {
  #[inline]
  fn as_ref(&self) -> &[T] {
    &self.items
  }
}

impl<'a, T: Ord> IntoIterator for &'a SortedList<T> {
  type Item = &'a T;
  type IntoIter = core::slice::Iter<'a, T>;

  #[inline]
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

impl ListFormatter for FixedStr {
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
      acc.push_str(&item.to_human_readable_string());
      acc.push('"');
    }

    acc.push(']');
    acc
  }
}
