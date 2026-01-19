use crate::validators::builder_internals::*;
use proc_macro_impls::builder_state_macro;
builder_state_macro!(Ignore, Required, In, NotIn, Const, Lt, Lte, Gt, Gte);
