# Enum Reference

## Container attributes:

- `reserved_numbers`
    - Type: list of individual numbers or closed ranges
    - Example: `#[proto(reserved_numbers(1, 2..10, 5000..MAX))]`
    - Description:
        Specifies the reserved numbers for the given enum. These will be skipped when automatically generating tags for each field, and copied as such in the proto files output. In order to reserve up to the maximum tag range, use the `MAX` ident as shown above.

- `reserved_names`
    - Type: list of strings
    - Example: `#[proto(reserved_names("MYENUM_ABC", "MYENUM_DEG"))]`
    - Description:
        Specifies the reserved names for the given enum

- `options`
    - Type: Expr
    - Example: `#[proto(options = vec![ my_option_1() ])]`
    - Description:
        Specifies the options for the given enum. It must resolve to an implementor of IntoIterator<Item = [`ProtoOption`](crate::ProtoOption).

- `name`
    - Type: string
    - Example: `#[proto(name = "MyEnum")]`
    - Description:
        Specifies the name of the message. Overrides the default behaviour, which uses the name of the rust enum.

- `parent_message`
    - Type: Ident
    - Example: `#[proto(parent_message = MyMsg)]`
    - Description:
        Specifies the parent message of a nested enum.

- `deprecated`
    - Type: Ident
    - Example: `#[proto(deprecated = false)]` or `#[deprecated]`
    - Description:
    Marks the enum as deprecated. The proto output will reflect this setting.
    If for some reason you want to deprecate the rust enum but not the proto enum, you can use `#[deprecated]` along with `#[proto(deprecated = false)]`. Vice versa, you can deprecate the proto enum only by using `#[proto(deprecated = true)]`

## Variant Attributes

- `options`
    - Type: Expr
    - Example: `#[proto(options = vec![ my_option_1() ])]`
    - Description:
        Specifies the options for the given variant. It must resolve to an implementor of IntoIterator<Item = [`ProtoOption`](crate::ProtoOption).

- `tag`
    - Type: number
    - Example: `#[proto(tag = 10000)]`
    - Description:
    Specifies the protobuf tag for the given field. Unline in messages or enums, tags must be set manually for extensions.

- `name`
    - Type: string
    - Example: `#[proto(name = "MY_ENUM_ABC")]`
    - Description:
        Specifies the name for the given variant. Defaults to the SCREAMING_CASE name of the variant, prefixed by the name of the enum as per the proto convention.

- `deprecated`
    - Type: Ident
    - Example: `#[proto(deprecated = false)]` or `#[deprecated]`
    - Description:
        Marks the variant as deprecated. The proto output will reflect this setting.
        If for some reason you want to deprecate the rust variant but not the proto variant, you can use `#[deprecated]` along with `#[proto(deprecated = false)]`. Vice versa, you can deprecate the proto variant only by using `#[proto(deprecated = true)]`
