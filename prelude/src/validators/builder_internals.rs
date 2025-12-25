use crate::*;

#[derive(Default)]
pub struct Empty;

pub trait IsUnset {}

#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct Set<T>(PhantomData<fn() -> T>);
#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct Unset<T>(PhantomData<fn() -> T>);

#[doc(hidden)]
impl<T> IsUnset for Unset<T> {}
