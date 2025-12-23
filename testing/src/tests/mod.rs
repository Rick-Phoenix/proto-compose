mod cel_checks_tests;
mod maps_tests;
mod oneof_tags_tests;
mod rendering_tests;
mod repeated_tests;

use std::collections::HashMap;

use bytes::Bytes;
use prelude::*;
use proc_macro_impls::*;

proto_file!("testing", package = "testing");
