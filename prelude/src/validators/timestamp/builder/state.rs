#![allow(private_interfaces)]

use std::marker::PhantomData;

use crate::validators::builder_internals::*;

mod sealed {
  pub(super) struct Sealed;
}

pub trait State: Sized {
  type Ignore;
  type LtNow;
  type GtNow;
  type Required;
  type Const;
  type Lt;
  type Lte;
  type Gt;
  type Gte;
  type Within;
  #[doc(hidden)]
  const SEALED: sealed::Sealed;
}

#[doc(hidden)]
#[allow(non_camel_case_types)]
mod members {
  pub struct ignore;
  pub struct lt_now;
  pub struct gt_now;
  pub struct required;
  pub struct const_;
  pub struct lt;
  pub struct lte;
  pub struct gt;
  pub struct gte;
  pub struct within;
}

pub struct SetIgnore<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetLtNow<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetGtNow<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetRequired<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetConst<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetLt<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetLte<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetGt<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetGte<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetWithin<S: State = Empty>(PhantomData<fn() -> S>);
#[doc(hidden)]
impl State for Empty {
  type Ignore = Unset<members::ignore>;
  type LtNow = Unset<members::lt_now>;
  type GtNow = Unset<members::gt_now>;
  type Required = Unset<members::required>;
  type Const = Unset<members::const_>;
  type Lt = Unset<members::lt>;
  type Lte = Unset<members::lte>;
  type Gt = Unset<members::gt>;
  type Gte = Unset<members::gte>;
  type Within = Unset<members::within>;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetIgnore<S> {
  type Ignore = Set<members::ignore>;
  type LtNow = S::LtNow;
  type GtNow = S::GtNow;
  type Required = S::Required;
  type Const = S::Const;
  type Lt = S::Lt;
  type Lte = S::Lte;
  type Gt = S::Gt;
  type Gte = S::Gte;
  type Within = S::Within;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetLtNow<S> {
  type Ignore = S::Ignore;
  type LtNow = Set<members::lt_now>;
  type GtNow = S::GtNow;
  type Required = S::Required;
  type Const = S::Const;
  type Lt = S::Lt;
  type Lte = S::Lte;
  type Gt = S::Gt;
  type Gte = S::Gte;
  type Within = S::Within;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetGtNow<S> {
  type Ignore = S::Ignore;
  type LtNow = S::LtNow;
  type GtNow = Set<members::gt_now>;
  type Required = S::Required;
  type Const = S::Const;
  type Lt = S::Lt;
  type Lte = S::Lte;
  type Gt = S::Gt;
  type Gte = S::Gte;
  type Within = S::Within;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetRequired<S> {
  type Ignore = S::Ignore;
  type LtNow = S::LtNow;
  type GtNow = S::GtNow;
  type Required = Set<members::required>;
  type Const = S::Const;
  type Lt = S::Lt;
  type Lte = S::Lte;
  type Gt = S::Gt;
  type Gte = S::Gte;
  type Within = S::Within;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetConst<S> {
  type Ignore = S::Ignore;
  type LtNow = S::LtNow;
  type GtNow = S::GtNow;
  type Required = S::Required;
  type Const = Set<members::const_>;
  type Lt = S::Lt;
  type Lte = S::Lte;
  type Gt = S::Gt;
  type Gte = S::Gte;
  type Within = S::Within;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetLt<S> {
  type Ignore = S::Ignore;
  type LtNow = S::LtNow;
  type GtNow = S::GtNow;
  type Required = S::Required;
  type Const = S::Const;
  type Lt = Set<members::lt>;
  type Lte = S::Lte;
  type Gt = S::Gt;
  type Gte = S::Gte;
  type Within = S::Within;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetLte<S> {
  type Ignore = S::Ignore;
  type LtNow = S::LtNow;
  type GtNow = S::GtNow;
  type Required = S::Required;
  type Const = S::Const;
  type Lt = S::Lt;
  type Lte = Set<members::lte>;
  type Gt = S::Gt;
  type Gte = S::Gte;
  type Within = S::Within;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetGt<S> {
  type Ignore = S::Ignore;
  type LtNow = S::LtNow;
  type GtNow = S::GtNow;
  type Required = S::Required;
  type Const = S::Const;
  type Lt = S::Lt;
  type Lte = S::Lte;
  type Gt = Set<members::gt>;
  type Gte = S::Gte;
  type Within = S::Within;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetGte<S> {
  type Ignore = S::Ignore;
  type LtNow = S::LtNow;
  type GtNow = S::GtNow;
  type Required = S::Required;
  type Const = S::Const;
  type Lt = S::Lt;
  type Lte = S::Lte;
  type Gt = S::Gt;
  type Gte = Set<members::gte>;
  type Within = S::Within;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetWithin<S> {
  type Ignore = S::Ignore;
  type LtNow = S::LtNow;
  type GtNow = S::GtNow;
  type Required = S::Required;
  type Const = S::Const;
  type Lt = S::Lt;
  type Lte = S::Lte;
  type Gt = S::Gt;
  type Gte = S::Gte;
  type Within = Set<members::within>;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
