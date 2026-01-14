#![allow(private_interfaces)]

use core::marker::PhantomData;

use crate::validators::builder_internals::*;

mod sealed {
  pub(super) struct Sealed;
}

mod members {
  pub struct Keys;
  pub struct Values;
  pub struct MinPairs;
  pub struct MaxPairs;
  pub struct Ignore;
  pub struct Cel;
}

pub trait State<S = Empty> {
  type Keys;
  type Values;
  type MinPairs;
  type MaxPairs;
  type Ignore;
  type Cel;
  const SEALED: sealed::Sealed;
}

pub struct SetKeys<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetValues<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetMinPairs<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetMaxPairs<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetIgnore<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetCel<S: State = Empty>(PhantomData<fn() -> S>);

#[doc(hidden)]
impl State for Empty {
  type Keys = Unset<members::Keys>;
  type Values = Unset<members::Values>;
  type MinPairs = Unset<members::MinPairs>;
  type MaxPairs = Unset<members::MaxPairs>;
  type Ignore = Unset<members::Ignore>;
  type Cel = Unset<members::Cel>;
  const SEALED: sealed::Sealed = sealed::Sealed;
}

#[doc(hidden)]
impl<S: State> State for SetCel<S> {
  type Keys = S::Keys;
  type Values = S::Values;
  type MinPairs = S::MinPairs;
  type MaxPairs = S::MaxPairs;
  type Ignore = S::Ignore;
  type Cel = Set<members::Cel>;
  const SEALED: sealed::Sealed = sealed::Sealed;
}

#[doc(hidden)]
impl<S: State> State for SetKeys<S> {
  type Keys = Set<members::Keys>;
  type Values = S::Values;
  type MinPairs = S::MinPairs;
  type MaxPairs = S::MaxPairs;
  type Ignore = S::Ignore;
  type Cel = S::Cel;
  const SEALED: sealed::Sealed = sealed::Sealed;
}

#[doc(hidden)]
impl<S: State> State for SetValues<S> {
  type Keys = S::Keys;
  type Values = Set<members::Values>;
  type MinPairs = S::MinPairs;
  type MaxPairs = S::MaxPairs;
  type Ignore = S::Ignore;
  type Cel = S::Cel;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetMinPairs<S> {
  type Keys = S::Keys;
  type Values = S::Values;
  type MinPairs = Set<members::MinPairs>;
  type MaxPairs = S::MaxPairs;
  type Ignore = S::Ignore;
  type Cel = S::Cel;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetMaxPairs<S> {
  type Keys = S::Keys;
  type Values = S::Values;
  type MinPairs = S::MinPairs;
  type MaxPairs = Set<members::MaxPairs>;
  type Ignore = S::Ignore;
  type Cel = S::Cel;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetIgnore<S> {
  type Keys = S::Keys;
  type Values = S::Values;
  type MinPairs = S::MinPairs;
  type MaxPairs = S::MaxPairs;
  type Ignore = Set<members::Ignore>;
  type Cel = S::Cel;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
