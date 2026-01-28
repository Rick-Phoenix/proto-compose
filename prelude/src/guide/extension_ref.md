# Extension Reference

## Macro Attributes

- `target`
    - Type: Ident representing a valid extension target
    - Example: `#[proto_extension(target = MessageOptions)]`
    - Description:
        The target of the given extension. It must match one of the supported targets from proto3 onwards, such as MessageOptions, FileOptions and so on.

## Field Attributes

- `options`
    - Type: Expr
    - Example: `#[proto(options = vec![ my_option_1() ])]`
    - Description:
        Specifies the options for the given field. It must resolve to an implementor of IntoIterator<Item = [`ProtoOption`](crate::ProtoOption).

- `tag`
    - Type: number
    - Example: `#[proto(tag = 10000)]`
    - Description:
        Specifies the protobuf tag for the given field. Unline in messages or enums, tags must be set manually for extensions.

- `name`
    - Type: string
    - Example: `#[proto(name = "abc")]`
    - Description:
        Specifies the name for the given field. Defaults to the name of the rust field.
