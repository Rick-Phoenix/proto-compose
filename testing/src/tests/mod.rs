mod cel_checks_tests;
mod maps_tests;
mod repeated_tests;

use std::collections::HashMap;

use bytes::Bytes;
use prelude::{cel_program, CachedProgram, Package, ProtoFile, ProtoMessage};
use proc_macro_impls::{proto_enum, proto_message};
