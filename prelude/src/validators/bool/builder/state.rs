#![allow(private_interfaces)]

use core::marker::PhantomData;

use crate::validators::builder_internals::*;

mod sealed {
  pub(super) struct Sealed;
}

pub trait State: Sized {
  type Const;
  type Required;
  type Ignore;
  #[doc(hidden)]
  const SEALED: sealed::Sealed;
}

#[doc(hidden)]
#[allow(non_camel_case_types)]
mod members {
  pub struct const_(());
  pub struct required(());
  pub struct ignore(());
}

pub struct SetConst<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetRequired<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetIgnore<S: State = Empty>(PhantomData<fn() -> S>);
#[doc(hidden)]
impl State for Empty {
  type Const = Unset<members::const_>;
  type Required = Unset<members::required>;
  type Ignore = Unset<members::ignore>;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetConst<S> {
  type Const = Set<members::const_>;
  type Required = S::Required;
  type Ignore = S::Ignore;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetRequired<S> {
  type Const = S::Const;
  type Required = Set<members::required>;
  type Ignore = S::Ignore;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetIgnore<S> {
  type Const = S::Const;
  type Required = S::Required;
  type Ignore = Set<members::ignore>;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
