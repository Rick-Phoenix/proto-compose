#[cfg(feature = "cel")]
use proto_types::cel::CelConversionError;

use crate::*;

#[doc(hidden)]
#[cfg(feature = "cel")]
pub trait IntoCel: Into<::cel::Value> {}
#[doc(hidden)]
#[cfg(feature = "cel")]
impl<T> IntoCel for T where T: Into<::cel::Value> {}

#[cfg(not(feature = "cel"))]
#[doc(hidden)]
pub trait IntoCel {}
#[cfg(not(feature = "cel"))]
#[doc(hidden)]
impl<T> IntoCel for T {}

#[cfg(feature = "cel")]
#[doc(hidden)]
pub trait TryIntoCel: Clone {
  fn __try_into_cel(self) -> Result<::cel::Value, CelError>;
}

#[cfg(feature = "cel")]
impl<T, E> TryIntoCel for T
where
  T: TryInto<::cel::Value, Error = E> + Clone,
  E: Display,
{
  #[inline]
  fn __try_into_cel(self) -> Result<::cel::Value, CelError> {
    self
      .try_into()
      .map_err(|e| CelError::ConversionError(e.to_string()))
  }
}

#[cfg(feature = "cel")]
pub trait CelValue: Clone + TryInto<::cel::Value, Error = CelConversionError> {
  #[inline]
  fn try_into_cel(self) -> Result<::cel::Value, CelConversionError> {
    self.try_into()
  }
}

#[cfg(feature = "cel")]
pub trait CelOneof: Sized + TryInto<::cel::Value, Error = CelConversionError> {
  fn try_into_cel(self) -> Result<(String, ::cel::Value), CelConversionError>;
}

#[cfg(not(feature = "cel"))]
#[doc(hidden)]
pub trait CelOneof {}
#[cfg(not(feature = "cel"))]
#[doc(hidden)]
impl<T> CelOneof for T {}

#[cfg(not(feature = "cel"))]
#[doc(hidden)]
pub trait CelValue {}
#[cfg(not(feature = "cel"))]
#[doc(hidden)]
impl<T> CelValue for T {}

#[cfg(not(feature = "cel"))]
#[doc(hidden)]
pub trait TryIntoCel {}
#[cfg(not(feature = "cel"))]
#[doc(hidden)]
impl<T> TryIntoCel for T {}

#[doc(hidden)]
#[cfg(feature = "cel")]
pub trait IntoCelKey: Into<::cel::objects::Key> {}
#[doc(hidden)]
#[cfg(feature = "cel")]
impl<T> IntoCelKey for T where T: Into<::cel::objects::Key> {}

#[doc(hidden)]
#[cfg(not(feature = "cel"))]
pub trait IntoCelKey {}
#[doc(hidden)]
#[cfg(not(feature = "cel"))]
impl<T> IntoCelKey for T {}
