use crate::validators::builder_internals::*;
use proc_macro_impls::builder_state_macro;
builder_state_macro!(
  Const,
  Required,
  Ignore,
  LtNow,
  Lt,
  Lte,
  GtNow,
  Gt,
  Gte,
  Within,
  NowTolerance
);
