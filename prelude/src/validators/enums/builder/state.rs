#![allow(private_interfaces)]

use core::marker::PhantomData;

use crate::validators::builder_internals::*;

mod sealed {
  pub(super) struct Sealed;
}

pub trait State: Sized {
  type Ignore;
  type DefinedOnly;
  type Required;
  type In;
  type NotIn;
  type Const;
  #[doc(hidden)]
  const SEALED: sealed::Sealed;
}

#[doc(hidden)]
#[allow(non_camel_case_types)]
mod members {
  pub struct ignore;
  pub struct defined_only;
  pub struct required;
  pub struct in_;
  pub struct not_in;
  pub struct const_;
}
pub struct SetIgnore<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetDefinedOnly<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetRequired<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetIn<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetNotIn<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetConst<S: State = Empty>(PhantomData<fn() -> S>);
#[doc(hidden)]
impl State for Empty {
  type Ignore = Unset<members::ignore>;
  type DefinedOnly = Unset<members::defined_only>;
  type Required = Unset<members::required>;
  type In = Unset<members::in_>;
  type NotIn = Unset<members::not_in>;
  type Const = Unset<members::const_>;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetIgnore<S> {
  type Ignore = Set<members::ignore>;
  type DefinedOnly = S::DefinedOnly;
  type Required = S::Required;
  type In = S::In;
  type NotIn = S::NotIn;
  type Const = S::Const;
  const SEALED: sealed::Sealed = sealed::Sealed;
}

#[doc(hidden)]
impl<S: State> State for SetDefinedOnly<S> {
  type Ignore = S::Ignore;
  type DefinedOnly = Set<members::defined_only>;
  type Required = S::Required;
  type In = S::In;
  type NotIn = S::NotIn;
  type Const = S::Const;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetRequired<S> {
  type Ignore = S::Ignore;
  type DefinedOnly = S::DefinedOnly;
  type Required = Set<members::required>;
  type In = S::In;
  type NotIn = S::NotIn;
  type Const = S::Const;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetIn<S> {
  type Ignore = S::Ignore;
  type DefinedOnly = S::DefinedOnly;
  type Required = S::Required;
  type In = Set<members::in_>;
  type NotIn = S::NotIn;
  type Const = S::Const;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetNotIn<S> {
  type Ignore = S::Ignore;
  type DefinedOnly = S::DefinedOnly;
  type Required = S::Required;
  type In = S::In;
  type NotIn = Set<members::not_in>;
  type Const = S::Const;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetConst<S> {
  type Ignore = S::Ignore;
  type DefinedOnly = S::DefinedOnly;
  type Required = S::Required;
  type In = S::In;
  type NotIn = S::NotIn;
  type Const = Set<members::const_>;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
