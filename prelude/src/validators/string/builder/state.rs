use crate::validators::builder_internals::*;
use proc_macro_impls::builder_state_macro;
builder_state_macro!(
  Const,
  Required,
  Ignore,
  WellKnown,
  Len,
  MinLen,
  MaxLen,
  LenBytes,
  MinBytes,
  MaxBytes,
  Pattern,
  Prefix,
  Suffix,
  NotContains,
  Contains,
  In,
  NotIn,
  ErrorMessages
);
