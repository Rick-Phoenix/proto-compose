#![allow(private_interfaces)]

use std::marker::PhantomData;

use crate::validators::builder_internals::*;

mod sealed {
  pub(super) struct Sealed;
}

pub trait State: Sized {
  type WellKnown;
  type Ignore;
  type Required;
  type Len;
  type MinLen;
  type MaxLen;
  type LenBytes;
  type MinBytes;
  type MaxBytes;
  type Pattern;
  type Prefix;
  type Suffix;
  type Contains;
  type NotContains;
  type In;
  type NotIn;
  type Const;
  #[doc(hidden)]
  const SEALED: sealed::Sealed;
}

#[doc(hidden)]
#[allow(non_camel_case_types)]
mod members {
  pub struct well_known;
  pub struct ignore;
  pub struct required;
  pub struct len;
  pub struct min_len;
  pub struct max_len;
  pub struct len_bytes;
  pub struct min_bytes;
  pub struct max_bytes;
  pub struct pattern;
  pub struct prefix;
  pub struct suffix;
  pub struct contains;
  pub struct not_contains;
  pub struct in_;
  pub struct not_in;
  pub struct const_;
}

pub struct SetWellKnown<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetIgnore<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetRequired<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetLen<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetMinLen<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetMaxLen<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetLenBytes<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetMinBytes<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetMaxBytes<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetPattern<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetPrefix<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetSuffix<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetContains<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetNotContains<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetIn<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetNotIn<S: State = Empty>(PhantomData<fn() -> S>);
pub struct SetConst<S: State = Empty>(PhantomData<fn() -> S>);
#[doc(hidden)]
impl State for Empty {
  type WellKnown = Unset<members::well_known>;
  type Ignore = Unset<members::ignore>;
  type Required = Unset<members::required>;
  type Len = Unset<members::len>;
  type MinLen = Unset<members::min_len>;
  type MaxLen = Unset<members::max_len>;
  type LenBytes = Unset<members::len_bytes>;
  type MinBytes = Unset<members::min_bytes>;
  type MaxBytes = Unset<members::max_bytes>;
  type Pattern = Unset<members::pattern>;
  type Prefix = Unset<members::prefix>;
  type Suffix = Unset<members::suffix>;
  type Contains = Unset<members::contains>;
  type NotContains = Unset<members::not_contains>;
  type In = Unset<members::in_>;
  type NotIn = Unset<members::not_in>;
  type Const = Unset<members::const_>;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetWellKnown<S> {
  type WellKnown = Set<members::well_known>;
  type Ignore = S::Ignore;
  type Required = S::Required;
  type Len = S::Len;
  type MinLen = S::MinLen;
  type MaxLen = S::MaxLen;
  type LenBytes = S::LenBytes;
  type MinBytes = S::MinBytes;
  type MaxBytes = S::MaxBytes;
  type Pattern = S::Pattern;
  type Prefix = S::Prefix;
  type Suffix = S::Suffix;
  type Contains = S::Contains;
  type NotContains = S::NotContains;
  type In = S::In;
  type NotIn = S::NotIn;
  type Const = S::Const;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetIgnore<S> {
  type WellKnown = S::WellKnown;
  type Ignore = Set<members::ignore>;
  type Required = S::Required;
  type Len = S::Len;
  type MinLen = S::MinLen;
  type MaxLen = S::MaxLen;
  type LenBytes = S::LenBytes;
  type MinBytes = S::MinBytes;
  type MaxBytes = S::MaxBytes;
  type Pattern = S::Pattern;
  type Prefix = S::Prefix;
  type Suffix = S::Suffix;
  type Contains = S::Contains;
  type NotContains = S::NotContains;
  type In = S::In;
  type NotIn = S::NotIn;
  type Const = S::Const;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetRequired<S> {
  type WellKnown = S::WellKnown;
  type Ignore = S::Ignore;
  type Required = Set<members::required>;
  type Len = S::Len;
  type MinLen = S::MinLen;
  type MaxLen = S::MaxLen;
  type LenBytes = S::LenBytes;
  type MinBytes = S::MinBytes;
  type MaxBytes = S::MaxBytes;
  type Pattern = S::Pattern;
  type Prefix = S::Prefix;
  type Suffix = S::Suffix;
  type Contains = S::Contains;
  type NotContains = S::NotContains;
  type In = S::In;
  type NotIn = S::NotIn;
  type Const = S::Const;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetLen<S> {
  type WellKnown = S::WellKnown;
  type Ignore = S::Ignore;
  type Required = S::Required;
  type Len = Set<members::len>;
  type MinLen = S::MinLen;
  type MaxLen = S::MaxLen;
  type LenBytes = S::LenBytes;
  type MinBytes = S::MinBytes;
  type MaxBytes = S::MaxBytes;
  type Pattern = S::Pattern;
  type Prefix = S::Prefix;
  type Suffix = S::Suffix;
  type Contains = S::Contains;
  type NotContains = S::NotContains;
  type In = S::In;
  type NotIn = S::NotIn;
  type Const = S::Const;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetMinLen<S> {
  type WellKnown = S::WellKnown;
  type Ignore = S::Ignore;
  type Required = S::Required;
  type Len = S::Len;
  type MinLen = Set<members::min_len>;
  type MaxLen = S::MaxLen;
  type LenBytes = S::LenBytes;
  type MinBytes = S::MinBytes;
  type MaxBytes = S::MaxBytes;
  type Pattern = S::Pattern;
  type Prefix = S::Prefix;
  type Suffix = S::Suffix;
  type Contains = S::Contains;
  type NotContains = S::NotContains;
  type In = S::In;
  type NotIn = S::NotIn;
  type Const = S::Const;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetMaxLen<S> {
  type WellKnown = S::WellKnown;
  type Ignore = S::Ignore;
  type Required = S::Required;
  type Len = S::Len;
  type MinLen = S::MinLen;
  type MaxLen = Set<members::max_len>;
  type LenBytes = S::LenBytes;
  type MinBytes = S::MinBytes;
  type MaxBytes = S::MaxBytes;
  type Pattern = S::Pattern;
  type Prefix = S::Prefix;
  type Suffix = S::Suffix;
  type Contains = S::Contains;
  type NotContains = S::NotContains;
  type In = S::In;
  type NotIn = S::NotIn;
  type Const = S::Const;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetLenBytes<S> {
  type WellKnown = S::WellKnown;
  type Ignore = S::Ignore;
  type Required = S::Required;
  type Len = S::Len;
  type MinLen = S::MinLen;
  type MaxLen = S::MaxLen;
  type LenBytes = Set<members::len_bytes>;
  type MinBytes = S::MinBytes;
  type MaxBytes = S::MaxBytes;
  type Pattern = S::Pattern;
  type Prefix = S::Prefix;
  type Suffix = S::Suffix;
  type Contains = S::Contains;
  type NotContains = S::NotContains;
  type In = S::In;
  type NotIn = S::NotIn;
  type Const = S::Const;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetMinBytes<S> {
  type WellKnown = S::WellKnown;
  type Ignore = S::Ignore;
  type Required = S::Required;
  type Len = S::Len;
  type MinLen = S::MinLen;
  type MaxLen = S::MaxLen;
  type LenBytes = S::LenBytes;
  type MinBytes = Set<members::min_bytes>;
  type MaxBytes = S::MaxBytes;
  type Pattern = S::Pattern;
  type Prefix = S::Prefix;
  type Suffix = S::Suffix;
  type Contains = S::Contains;
  type NotContains = S::NotContains;
  type In = S::In;
  type NotIn = S::NotIn;
  type Const = S::Const;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetMaxBytes<S> {
  type WellKnown = S::WellKnown;
  type Ignore = S::Ignore;
  type Required = S::Required;
  type Len = S::Len;
  type MinLen = S::MinLen;
  type MaxLen = S::MaxLen;
  type LenBytes = S::LenBytes;
  type MinBytes = S::MinBytes;
  type MaxBytes = Set<members::max_bytes>;
  type Pattern = S::Pattern;
  type Prefix = S::Prefix;
  type Suffix = S::Suffix;
  type Contains = S::Contains;
  type NotContains = S::NotContains;
  type In = S::In;
  type NotIn = S::NotIn;
  type Const = S::Const;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetPattern<S> {
  type WellKnown = S::WellKnown;
  type Ignore = S::Ignore;
  type Required = S::Required;
  type Len = S::Len;
  type MinLen = S::MinLen;
  type MaxLen = S::MaxLen;
  type LenBytes = S::LenBytes;
  type MinBytes = S::MinBytes;
  type MaxBytes = S::MaxBytes;
  type Pattern = Set<members::pattern>;
  type Prefix = S::Prefix;
  type Suffix = S::Suffix;
  type Contains = S::Contains;
  type NotContains = S::NotContains;
  type In = S::In;
  type NotIn = S::NotIn;
  type Const = S::Const;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetPrefix<S> {
  type WellKnown = S::WellKnown;
  type Ignore = S::Ignore;
  type Required = S::Required;
  type Len = S::Len;
  type MinLen = S::MinLen;
  type MaxLen = S::MaxLen;
  type LenBytes = S::LenBytes;
  type MinBytes = S::MinBytes;
  type MaxBytes = S::MaxBytes;
  type Pattern = S::Pattern;
  type Prefix = Set<members::prefix>;
  type Suffix = S::Suffix;
  type Contains = S::Contains;
  type NotContains = S::NotContains;
  type In = S::In;
  type NotIn = S::NotIn;
  type Const = S::Const;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetSuffix<S> {
  type WellKnown = S::WellKnown;
  type Ignore = S::Ignore;
  type Required = S::Required;
  type Len = S::Len;
  type MinLen = S::MinLen;
  type MaxLen = S::MaxLen;
  type LenBytes = S::LenBytes;
  type MinBytes = S::MinBytes;
  type MaxBytes = S::MaxBytes;
  type Pattern = S::Pattern;
  type Prefix = S::Prefix;
  type Suffix = Set<members::suffix>;
  type Contains = S::Contains;
  type NotContains = S::NotContains;
  type In = S::In;
  type NotIn = S::NotIn;
  type Const = S::Const;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetContains<S> {
  type WellKnown = S::WellKnown;
  type Ignore = S::Ignore;
  type Required = S::Required;
  type Len = S::Len;
  type MinLen = S::MinLen;
  type MaxLen = S::MaxLen;
  type LenBytes = S::LenBytes;
  type MinBytes = S::MinBytes;
  type MaxBytes = S::MaxBytes;
  type Pattern = S::Pattern;
  type Prefix = S::Prefix;
  type Suffix = S::Suffix;
  type Contains = Set<members::contains>;
  type NotContains = S::NotContains;
  type In = S::In;
  type NotIn = S::NotIn;
  type Const = S::Const;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetNotContains<S> {
  type WellKnown = S::WellKnown;
  type Ignore = S::Ignore;
  type Required = S::Required;
  type Len = S::Len;
  type MinLen = S::MinLen;
  type MaxLen = S::MaxLen;
  type LenBytes = S::LenBytes;
  type MinBytes = S::MinBytes;
  type MaxBytes = S::MaxBytes;
  type Pattern = S::Pattern;
  type Prefix = S::Prefix;
  type Suffix = S::Suffix;
  type Contains = S::Contains;
  type NotContains = Set<members::not_contains>;
  type In = S::In;
  type NotIn = S::NotIn;
  type Const = S::Const;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetIn<S> {
  type WellKnown = S::WellKnown;
  type Ignore = S::Ignore;
  type Required = S::Required;
  type Len = S::Len;
  type MinLen = S::MinLen;
  type MaxLen = S::MaxLen;
  type LenBytes = S::LenBytes;
  type MinBytes = S::MinBytes;
  type MaxBytes = S::MaxBytes;
  type Pattern = S::Pattern;
  type Prefix = S::Prefix;
  type Suffix = S::Suffix;
  type Contains = S::Contains;
  type NotContains = S::NotContains;
  type In = Set<members::in_>;
  type NotIn = S::NotIn;
  type Const = S::Const;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetNotIn<S> {
  type WellKnown = S::WellKnown;
  type Ignore = S::Ignore;
  type Required = S::Required;
  type Len = S::Len;
  type MinLen = S::MinLen;
  type MaxLen = S::MaxLen;
  type LenBytes = S::LenBytes;
  type MinBytes = S::MinBytes;
  type MaxBytes = S::MaxBytes;
  type Pattern = S::Pattern;
  type Prefix = S::Prefix;
  type Suffix = S::Suffix;
  type Contains = S::Contains;
  type NotContains = S::NotContains;
  type In = S::In;
  type NotIn = Set<members::not_in>;
  type Const = S::Const;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
#[doc(hidden)]
impl<S: State> State for SetConst<S> {
  type WellKnown = S::WellKnown;
  type Ignore = S::Ignore;
  type Required = S::Required;
  type Len = S::Len;
  type MinLen = S::MinLen;
  type MaxLen = S::MaxLen;
  type LenBytes = S::LenBytes;
  type MinBytes = S::MinBytes;
  type MaxBytes = S::MaxBytes;
  type Pattern = S::Pattern;
  type Prefix = S::Prefix;
  type Suffix = S::Suffix;
  type Contains = S::Contains;
  type NotContains = S::NotContains;
  type In = S::In;
  type NotIn = S::NotIn;
  type Const = Set<members::const_>;
  const SEALED: sealed::Sealed = sealed::Sealed;
}
