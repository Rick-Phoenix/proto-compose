# Service Reference

Services are defined via the `#[proto_service]` macro, which should be applied to an enum which contains variants that have two named fields, `request` and `response`. The following attributes can be applied both to the service as a whole and to its variants.

- `options`
    - Type: Expr
    - Example: `#[proto(options = vec![ my_option_1() ])]`
    - Description:
        Specifies the options for the given service/handler. It must resolve to an implementor of IntoIterator<Item = [`ProtoOption`](crate::ProtoOption).

- `deprecated`
    - Type: Ident
    - Example: `#[proto(deprecated = true)]` or `#[deprecated]`
    - Description:
        Marks the service/handler as deprecated. The proto output will reflect this setting.
