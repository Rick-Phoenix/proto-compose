#![allow(private_interfaces)]

use std::marker::PhantomData;

use crate::validators::builder_internals::*;

mod sealed {
  pub(super) struct Sealed;
}

pub trait State: Sized {
  type Ignore;
  type Required;
  type In;
  type NotIn;
  #[doc(hidden)]
  const SEALED: sealed::Sealed;
}

#[allow(non_camel_case_types)]
mod members {
  pub struct ignore;
  pub struct required;
  pub struct in_;
  pub struct not_in;
}
pub struct SetIgnore<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetRequired<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetIn<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetNotIn<S: State = Empty>(PhantomData<fn() -> S>);
#[doc(hidden)]
impl State for Empty {
  type Ignore = Unset<members::ignore>;
  type Required = Unset<members::required>;
  type In = Unset<members::in_>;
  type NotIn = Unset<members::not_in>;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetIgnore<S> {
  type Ignore = Set<members::ignore>;
  type Required = S::Required;
  type In = S::In;
  type NotIn = S::NotIn;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetRequired<S> {
  type Ignore = S::Ignore;
  type Required = Set<members::required>;
  type In = S::In;
  type NotIn = S::NotIn;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetIn<S> {
  type Ignore = S::Ignore;
  type Required = S::Required;
  type In = Set<members::in_>;
  type NotIn = S::NotIn;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetNotIn<S> {
  type Ignore = S::Ignore;
  type Required = S::Required;
  type In = S::In;
  type NotIn = Set<members::not_in>;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
