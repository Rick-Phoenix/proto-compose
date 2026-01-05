use std::{fs, sync::LazyLock};

use prost_reflect::{DescriptorPool, ExtensionDescriptor};

pub static DESCRIPTOR_POOL: LazyLock<DescriptorPool> = LazyLock::new(|| {
  let descriptor_set_path = std::env::var("PROTO_DESCRIPTOR_SET")
        .expect("PROTO_DESCRIPTOR_SET environment variable not set. This is required to access Protobuf schema information.");
  let descriptor_set_bytes = fs::read(&descriptor_set_path)
    .unwrap_or_else(|e| panic!("Failed to read descriptor set from {descriptor_set_path}: {e}"));
  DescriptorPool::decode(descriptor_set_bytes.as_slice())
        .expect("Failed to decode DescriptorPool from descriptor set bytes. Ensure your protobuf definitions are valid.")
});

pub static FIELD_RULES_EXT_DESCRIPTOR: LazyLock<ExtensionDescriptor> = LazyLock::new(|| {
  DESCRIPTOR_POOL
    .get_extension_by_name("buf.validate.field")
    .expect("buf.validate.field extension not found in descriptor pool")
});

pub static ONEOF_RULES_EXT_DESCRIPTOR: LazyLock<ExtensionDescriptor> = LazyLock::new(|| {
  DESCRIPTOR_POOL
    .get_extension_by_name("buf.validate.oneof")
    .expect("buf.validate.oneof extension not found in descriptor pool")
});

pub static MESSAGE_RULES_EXT_DESCRIPTOR: LazyLock<ExtensionDescriptor> = LazyLock::new(|| {
  DESCRIPTOR_POOL
    .get_extension_by_name("buf.validate.message")
    .expect("buf.validate.message extension not found in descriptor pool")
});
