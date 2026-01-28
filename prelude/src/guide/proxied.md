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
