use crate::validators::builder_internals::*;
use proc_macro_impls::builder_state_macro;
builder_state_macro!(
  Const,
  Required,
  Ignore,
  AbsTolerance,
  RelTolerance,
  Finite,
  Lt,
  Lte,
  Gt,
  Gte,
  In,
  NotIn
);
