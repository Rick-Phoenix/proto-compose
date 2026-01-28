use alloc::boxed::Box;
use once_cell::race::OnceBox;

use crate::*;

/// A lazy initializer, to use for static validators.
/// It functions as a wrapper to [`OnceBox`](once_cell::race::OnceBox) that initializes at first deref like `LazyLock`.
pub struct Lazy<T, F = fn() -> T> {
  cell: OnceBox<T>,
  init: F,
}

impl<T, F> Lazy<T, F>
where
  F: Fn() -> T,
{
  /// Creates a new instance.
  pub const fn new(f: F) -> Self {
    Self {
      cell: OnceBox::new(),
      init: f,
    }
  }

  #[inline(never)]
  #[cold]
  fn start(&self) -> Box<T> {
    Box::new((self.init)())
  }
}

impl<T, F: Fn() -> T> Deref for Lazy<T, F> {
  type Target = T;

  #[inline]
  fn deref(&self) -> &T {
    self.cell.get_or_init(|| self.start())
  }
}
