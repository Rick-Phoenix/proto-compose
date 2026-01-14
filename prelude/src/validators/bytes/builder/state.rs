#![allow(private_interfaces)]

use core::marker::PhantomData;

use crate::validators::builder_internals::*;

mod sealed {
  pub(super) struct Sealed;
}

pub trait State: Sized {
  type Ignore;
  type WellKnown;
  type Required;
  type Len;
  type MinLen;
  type MaxLen;
  type Pattern;
  type Prefix;
  type Suffix;
  type Contains;
  type In;
  type NotIn;
  type Const;
  #[doc(hidden)]
  const SEALED: sealed::Sealed;
}

#[doc(hidden)]
#[allow(non_camel_case_types)]
mod members {
  pub struct ignore(());
  pub struct well_known(());
  pub struct required(());
  pub struct len(());
  pub struct min_len(());
  pub struct max_len(());
  pub struct pattern(());
  pub struct prefix(());
  pub struct suffix(());
  pub struct contains(());
  pub struct in_(());
  pub struct not_in(());
  pub struct const_(());
}

pub struct SetIgnore<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetWellKnown<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetRequired<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetLen<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetMinLen<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetMaxLen<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetPattern<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetPrefix<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetSuffix<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetContains<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetIn<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetNotIn<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetConst<S: State = Empty>(PhantomData<fn() -> S>);
#[doc(hidden)]
impl State for Empty {
  type Ignore = Unset<members::ignore>;
  type WellKnown = Unset<members::well_known>;
  type Required = Unset<members::required>;
  type Len = Unset<members::len>;
  type MinLen = Unset<members::min_len>;
  type MaxLen = Unset<members::max_len>;
  type Pattern = Unset<members::pattern>;
  type Prefix = Unset<members::prefix>;
  type Suffix = Unset<members::suffix>;
  type Contains = Unset<members::contains>;
  type In = Unset<members::in_>;
  type NotIn = Unset<members::not_in>;
  type Const = Unset<members::const_>;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetIgnore<S> {
  type Ignore = Set<members::ignore>;
  type WellKnown = S::WellKnown;
  type Required = S::Required;
  type Len = S::Len;
  type MinLen = S::MinLen;
  type MaxLen = S::MaxLen;
  type Pattern = S::Pattern;
  type Prefix = S::Prefix;
  type Suffix = S::Suffix;
  type Contains = S::Contains;
  type In = S::In;
  type NotIn = S::NotIn;
  type Const = S::Const;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetWellKnown<S> {
  type Ignore = S::Ignore;
  type WellKnown = Set<members::well_known>;
  type Required = S::Required;
  type Len = S::Len;
  type MinLen = S::MinLen;
  type MaxLen = S::MaxLen;
  type Pattern = S::Pattern;
  type Prefix = S::Prefix;
  type Suffix = S::Suffix;
  type Contains = S::Contains;
  type In = S::In;
  type NotIn = S::NotIn;
  type Const = S::Const;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetRequired<S> {
  type Ignore = S::Ignore;
  type WellKnown = S::WellKnown;
  type Required = Set<members::required>;
  type Len = S::Len;
  type MinLen = S::MinLen;
  type MaxLen = S::MaxLen;
  type Pattern = S::Pattern;
  type Prefix = S::Prefix;
  type Suffix = S::Suffix;
  type Contains = S::Contains;
  type In = S::In;
  type NotIn = S::NotIn;
  type Const = S::Const;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetLen<S> {
  type Ignore = S::Ignore;
  type WellKnown = S::WellKnown;
  type Required = S::Required;
  type Len = Set<members::len>;
  type MinLen = S::MinLen;
  type MaxLen = S::MaxLen;
  type Pattern = S::Pattern;
  type Prefix = S::Prefix;
  type Suffix = S::Suffix;
  type Contains = S::Contains;
  type In = S::In;
  type NotIn = S::NotIn;
  type Const = S::Const;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetMinLen<S> {
  type Ignore = S::Ignore;
  type WellKnown = S::WellKnown;
  type Required = S::Required;
  type Len = S::Len;
  type MinLen = Set<members::min_len>;
  type MaxLen = S::MaxLen;
  type Pattern = S::Pattern;
  type Prefix = S::Prefix;
  type Suffix = S::Suffix;
  type Contains = S::Contains;
  type In = S::In;
  type NotIn = S::NotIn;
  type Const = S::Const;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetMaxLen<S> {
  type Ignore = S::Ignore;
  type WellKnown = S::WellKnown;
  type Required = S::Required;
  type Len = S::Len;
  type MinLen = S::MinLen;
  type MaxLen = Set<members::max_len>;
  type Pattern = S::Pattern;
  type Prefix = S::Prefix;
  type Suffix = S::Suffix;
  type Contains = S::Contains;
  type In = S::In;
  type NotIn = S::NotIn;
  type Const = S::Const;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetPattern<S> {
  type Ignore = S::Ignore;
  type WellKnown = S::WellKnown;
  type Required = S::Required;
  type Len = S::Len;
  type MinLen = S::MinLen;
  type MaxLen = S::MaxLen;
  type Pattern = Set<members::pattern>;
  type Prefix = S::Prefix;
  type Suffix = S::Suffix;
  type Contains = S::Contains;
  type In = S::In;
  type NotIn = S::NotIn;
  type Const = S::Const;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetPrefix<S> {
  type Ignore = S::Ignore;
  type WellKnown = S::WellKnown;
  type Required = S::Required;
  type Len = S::Len;
  type MinLen = S::MinLen;
  type MaxLen = S::MaxLen;
  type Pattern = S::Pattern;
  type Prefix = Set<members::prefix>;
  type Suffix = S::Suffix;
  type Contains = S::Contains;
  type In = S::In;
  type NotIn = S::NotIn;
  type Const = S::Const;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetSuffix<S> {
  type Ignore = S::Ignore;
  type WellKnown = S::WellKnown;
  type Required = S::Required;
  type Len = S::Len;
  type MinLen = S::MinLen;
  type MaxLen = S::MaxLen;
  type Pattern = S::Pattern;
  type Prefix = S::Prefix;
  type Suffix = Set<members::suffix>;
  type Contains = S::Contains;
  type In = S::In;
  type NotIn = S::NotIn;
  type Const = S::Const;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetContains<S> {
  type Ignore = S::Ignore;
  type WellKnown = S::WellKnown;
  type Required = S::Required;
  type Len = S::Len;
  type MinLen = S::MinLen;
  type MaxLen = S::MaxLen;
  type Pattern = S::Pattern;
  type Prefix = S::Prefix;
  type Suffix = S::Suffix;
  type Contains = Set<members::contains>;
  type In = S::In;
  type NotIn = S::NotIn;
  type Const = S::Const;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetIn<S> {
  type Ignore = S::Ignore;
  type WellKnown = S::WellKnown;
  type Required = S::Required;
  type Len = S::Len;
  type MinLen = S::MinLen;
  type MaxLen = S::MaxLen;
  type Pattern = S::Pattern;
  type Prefix = S::Prefix;
  type Suffix = S::Suffix;
  type Contains = S::Contains;
  type In = Set<members::in_>;
  type NotIn = S::NotIn;
  type Const = S::Const;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetNotIn<S> {
  type Ignore = S::Ignore;
  type WellKnown = S::WellKnown;
  type Required = S::Required;
  type Len = S::Len;
  type MinLen = S::MinLen;
  type MaxLen = S::MaxLen;
  type Pattern = S::Pattern;
  type Prefix = S::Prefix;
  type Suffix = S::Suffix;
  type Contains = S::Contains;
  type In = S::In;
  type NotIn = Set<members::not_in>;
  type Const = S::Const;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetConst<S> {
  type Ignore = S::Ignore;
  type WellKnown = S::WellKnown;
  type Required = S::Required;
  type Len = S::Len;
  type MinLen = S::MinLen;
  type MaxLen = S::MaxLen;
  type Pattern = S::Pattern;
  type Prefix = S::Prefix;
  type Suffix = S::Suffix;
  type Contains = S::Contains;
  type In = S::In;
  type NotIn = S::NotIn;
  type Const = Set<members::const_>;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
