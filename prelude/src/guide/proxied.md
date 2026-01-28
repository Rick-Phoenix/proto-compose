# Proxied impls

Messages and oneofs can be proxied. This will generate a new struct with the same name, followed by a `Proto` suffix.

Proxied messages/oneofs unlock the following features:

- A field/variant can be missing from the proto struct, but present in the proxy
- Enums can use their actual rust enums as the type, rather than pure integers
- Oneofs can be not optional and default to their default value
- Messages can be not optional and default to their default value
- Types that are not supported by prost can be used in the proxy

By default, the macro will generate a conversion from proxy to proto and vice versa that just calls .into(). To provide custom conversions, you can use the `from_proto` and `into_proto` attributes on the container (to replace the automatically generated impl as a whole) or on individual fields/variants.

The messages/oneofs will also implement [`ProxiedMessage`](crate::ProxiedMessage) or [`ProxiedOneof`](crate::ProxiedOneof), whereas the proxies will implement [`MessageProxy`](crate::MessageProxy) and [`OneofProxy`](crate::OneofProxy).

## Examples

```rust
use prelude::*;
use std::sync::Arc;

proto_package!(MY_PKG, name = "my_pkg");
define_proto_file!(MY_FILE, name = "my_file.proto", package = MY_PKG);


// Generates a MsgProto struct that is protobuf-compatible
#[proto_message(proxied)]
pub struct Msg {
    // Requires setting the type manually as the type
    // is not prost-compatible
    #[proto(string)]
    // Must provide a custom `into_proto` impl because `Arc<str>` does not support `Into<String>`
    #[proto(into_proto = |v| v.as_ref().to_string())]
    pub name: Arc<str>,
    // Ignored field. Conversion from proto will use `Default::default()` unless a custom
    // conversion is specified
    #[proto(ignore)]
    pub rust_only: i32,
    // In proxied messages, we can use `default` for oneofs
    // so that using `Option` is not required.
    // The default conversion will call `ProxiedOneofProto::default().into()`
    // if the field is `None` in the proto struct.
    #[proto(oneof(proxied, default, tags(1, 2)))]
    pub oneof: ProxiedOneof,
    // We can do the same for messages too
    #[proto(message(default))]
    pub message_with_default: Msg2,
    // We can use the enum directly as the type
    #[proto(enum_)]
    pub enum_: TestEnum
}

#[proto_enum]
pub enum TestEnum {
    Unspecified, A, B
}

#[proto_message]
pub struct Msg2 {
    pub id: i32,
    // In direct impls, enums are just integers
    #[proto(enum_(TestEnum))]
    pub enum_: i32
}

// Generates the `ProxiedOneofProto` enum
#[proto_oneof(proxied)]
pub enum ProxiedOneof {
    #[proto(string, tag = 1, into_proto = |v| v.as_ref().to_string())]
    A(Arc<str>),
    #[proto(tag = 2)]
    B(u32),
}

impl Default for ProxiedOneofProto {
    fn default() -> Self {
        Self::B(1)
    }
}

fn main() {
    let msg = MsgProto::default();
    // Using the `ProxiedMessage` trait
    let proxy = msg.into_proxy();
    // Using the `MessageProxy` trait
    let msg_again = proxy.into_message();
}
```
