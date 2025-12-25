#![allow(private_interfaces)]

use std::marker::PhantomData;

use crate::validators::builder_internals::*;

mod sealed {
  pub(super) struct Sealed;
}

pub trait State: ::core::marker::Sized {
  type Ignore;
  type Required;
  #[doc(hidden)]
  const SEALED: sealed::Sealed;
}

#[doc(hidden)]
#[allow(non_camel_case_types)]
mod members {
  pub struct ignore;
  pub struct message;
  pub struct required;
}

pub struct SetIgnore<S: State = Empty>(::core::marker::PhantomData<fn() -> S>);
pub struct SetRequired<S: State = Empty>(::core::marker::PhantomData<fn() -> S>);
#[doc(hidden)]
impl State for Empty {
  type Ignore = Unset<members::ignore>;
  type Required = Unset<members::required>;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetIgnore<S> {
  type Ignore = Set<members::ignore>;
  type Required = S::Required;
  const SEALED: sealed::Sealed = sealed::Sealed;
}

#[doc(hidden)]
impl<S: State> State for SetRequired<S> {
  type Ignore = S::Ignore;
  type Required = Set<members::required>;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
