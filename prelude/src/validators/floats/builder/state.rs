#![allow(private_interfaces)]

use std::marker::PhantomData;

use crate::validators::builder_internals::*;

mod sealed {
  pub(super) struct Sealed;
}

pub trait State: Sized {
  type Ignore;
  type Required;
  type AbsTolerance;
  type RelTolerance;
  type Finite;
  type Const;
  type Lt;
  type Lte;
  type Gt;
  type Gte;
  type In;
  type NotIn;
  #[doc(hidden)]
  const SEALED: sealed::Sealed;
}

#[allow(non_camel_case_types)]
mod members {
  pub struct ignore;
  pub struct required;
  pub struct abs_tolerance;
  pub struct rel_tolerance;
  pub struct finite;
  pub struct const_;
  pub struct lt;
  pub struct lte;
  pub struct gt;
  pub struct gte;
  pub struct in_;
  pub struct not_in;
}

pub struct SetIgnore<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetRequired<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetAbsTolerance<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetRelTolerance<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetFinite<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetConst<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetLt<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetLte<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetGt<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetGte<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetIn<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetNotIn<S: State = Empty>(PhantomData<fn() -> S>);

#[doc(hidden)]
impl State for Empty {
  type Ignore = Unset<members::ignore>;
  type Required = Unset<members::required>;
  type AbsTolerance = Unset<members::abs_tolerance>;
  type RelTolerance = Unset<members::rel_tolerance>;
  type Finite = Unset<members::finite>;
  type Const = Unset<members::const_>;
  type Lt = Unset<members::lt>;
  type Lte = Unset<members::lte>;
  type Gt = Unset<members::gt>;
  type Gte = Unset<members::gte>;
  type In = Unset<members::in_>;
  type NotIn = Unset<members::not_in>;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetIgnore<S> {
  type Ignore = Set<members::ignore>;
  type Required = S::Required;
  type AbsTolerance = S::AbsTolerance;
  type RelTolerance = S::RelTolerance;
  type Finite = S::Finite;
  type Const = S::Const;
  type Lt = S::Lt;
  type Lte = S::Lte;
  type Gt = S::Gt;
  type Gte = S::Gte;
  type In = S::In;
  type NotIn = S::NotIn;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetRequired<S> {
  type Ignore = S::Ignore;
  type Required = Set<members::required>;
  type AbsTolerance = S::AbsTolerance;
  type RelTolerance = S::RelTolerance;
  type Finite = S::Finite;
  type Const = S::Const;
  type Lt = S::Lt;
  type Lte = S::Lte;
  type Gt = S::Gt;
  type Gte = S::Gte;
  type In = S::In;
  type NotIn = S::NotIn;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetAbsTolerance<S> {
  type Ignore = S::Ignore;
  type Required = S::Required;
  type AbsTolerance = Set<members::abs_tolerance>;
  type RelTolerance = S::RelTolerance;
  type Finite = S::Finite;
  type Const = S::Const;
  type Lt = S::Lt;
  type Lte = S::Lte;
  type Gt = S::Gt;
  type Gte = S::Gte;
  type In = S::In;
  type NotIn = S::NotIn;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetRelTolerance<S> {
  type Ignore = S::Ignore;
  type Required = S::Required;
  type AbsTolerance = S::AbsTolerance;
  type RelTolerance = Set<members::rel_tolerance>;
  type Finite = S::Finite;
  type Const = S::Const;
  type Lt = S::Lt;
  type Lte = S::Lte;
  type Gt = S::Gt;
  type Gte = S::Gte;
  type In = S::In;
  type NotIn = S::NotIn;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetFinite<S> {
  type Ignore = S::Ignore;
  type Required = S::Required;
  type AbsTolerance = S::AbsTolerance;
  type RelTolerance = S::RelTolerance;
  type Finite = Set<members::finite>;
  type Const = S::Const;
  type Lt = S::Lt;
  type Lte = S::Lte;
  type Gt = S::Gt;
  type Gte = S::Gte;
  type In = S::In;
  type NotIn = S::NotIn;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetConst<S> {
  type Ignore = S::Ignore;
  type Required = S::Required;
  type AbsTolerance = S::AbsTolerance;
  type RelTolerance = S::RelTolerance;
  type Finite = S::Finite;
  type Const = Set<members::const_>;
  type Lt = S::Lt;
  type Lte = S::Lte;
  type Gt = S::Gt;
  type Gte = S::Gte;
  type In = S::In;
  type NotIn = S::NotIn;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetLt<S> {
  type Ignore = S::Ignore;
  type Required = S::Required;
  type AbsTolerance = S::AbsTolerance;
  type RelTolerance = S::RelTolerance;
  type Finite = S::Finite;
  type Const = S::Const;
  type Lt = Set<members::lt>;
  type Lte = S::Lte;
  type Gt = S::Gt;
  type Gte = S::Gte;
  type In = S::In;
  type NotIn = S::NotIn;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetLte<S> {
  type Ignore = S::Ignore;
  type Required = S::Required;
  type AbsTolerance = S::AbsTolerance;
  type RelTolerance = S::RelTolerance;
  type Finite = S::Finite;
  type Const = S::Const;
  type Lt = S::Lt;
  type Lte = Set<members::lte>;
  type Gt = S::Gt;
  type Gte = S::Gte;
  type In = S::In;
  type NotIn = S::NotIn;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetGt<S> {
  type Ignore = S::Ignore;
  type Required = S::Required;
  type AbsTolerance = S::AbsTolerance;
  type RelTolerance = S::RelTolerance;
  type Finite = S::Finite;
  type Const = S::Const;
  type Lt = S::Lt;
  type Lte = S::Lte;
  type Gt = Set<members::gt>;
  type Gte = S::Gte;
  type In = S::In;
  type NotIn = S::NotIn;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetGte<S> {
  type Ignore = S::Ignore;
  type Required = S::Required;
  type AbsTolerance = S::AbsTolerance;
  type RelTolerance = S::RelTolerance;
  type Finite = S::Finite;
  type Const = S::Const;
  type Lt = S::Lt;
  type Lte = S::Lte;
  type Gt = S::Gt;
  type Gte = Set<members::gte>;
  type In = S::In;
  type NotIn = S::NotIn;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetIn<S> {
  type Ignore = S::Ignore;
  type Required = S::Required;
  type AbsTolerance = S::AbsTolerance;
  type RelTolerance = S::RelTolerance;
  type Finite = S::Finite;
  type Const = S::Const;
  type Lt = S::Lt;
  type Lte = S::Lte;
  type Gt = S::Gt;
  type Gte = S::Gte;
  type In = Set<members::in_>;
  type NotIn = S::NotIn;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetNotIn<S> {
  type Ignore = S::Ignore;
  type Required = S::Required;
  type AbsTolerance = S::AbsTolerance;
  type RelTolerance = S::RelTolerance;
  type Finite = S::Finite;
  type Const = S::Const;
  type Lt = S::Lt;
  type Lte = S::Lte;
  type Gt = S::Gt;
  type Gte = S::Gte;
  type In = S::In;
  type NotIn = Set<members::not_in>;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
