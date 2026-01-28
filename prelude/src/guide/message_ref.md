# Message Reference

## Macro attributes

- `proxied`
    - Type: Ident
    - Example: `#[proto_message(proxied)]`
    - Description:
        Creates a proxied message

## Container attributes:

- `derive`
    - Type: list of Paths
    - Example: `#[proto(derive(Copy))]`
    - Description:
        In case of a proxied impl, it specifies derives to apply to the Proto message.
        Shorthand for `#[proto(attr(derive(..)))]`

- `attr`
    - Type: MetaList
    - Example: `#[proto(attr(serde(default)))]`
    - Description:
        Forwards the given metas to the proto struct.
        In the example above, the proto struct would receive the attribute as `#[serde(default)]`.

- `reserved_numbers`
    - Type: list of individual numbers or closed ranges
    - Example: `#[proto(reserved_numbers(1, 2..10, 5000..MAX))]`
    - Description:
        Specifies the reserved numbers for the given message. These will be skipped when automatically generating tags for each field, and copied as such in the proto files output. In order to reserve up to the maximum tag range, use the `MAX` ident as shown above.

- `reserved_names`
    - Type: list of strings
    - Example: `#[proto(reserved_names("abc", "deg"))]`
    - Description:
        Specifies the reserved names for the given message

- `options`
    - Type: Expr
    - Example: `#[proto(options = vec![ my_option_1() ])]`
    - Description:
        Specifies the options for the given message. It must resolve to an implementor of IntoIterator<Item = [`ProtoOption`](crate::ProtoOption).

- `name`
    - Type: string
    - Example: `#[proto(name = "abc")]`
    - Description:
        Specifies the name of the message. Overrides the default behaviour, which uses the name of the struct.

- `parent_message`
    - Type: Ident
    - Example: `#[proto(parent_message = MyMsg)]`
    - Description:
        Specifies the parent message of a nested message.

- `from_proto`
    - Type: function Path or closure
    - Example: `#[proto(from_proto = my_convert_fn)]` or `#[proto(from_proto = |v| OtherType { val: v.val })]`
    - Description:
        Overrides the automatically generated conversion from proto to proxy.

- `into_proto`
    - Type: function Path or closure
    - Example: `#[proto(into_proto = my_convert_fn)]` or `#[proto(into_proto = |v| OtherType { val: v.val })]`
    - Description:
        Overrides the automatically generated conversion from proxy to proto.

- `deprecated`
    - Type: Ident
    - Example: `#[proto(deprecated = false)]` or `#[deprecated]`
    - Description:
        Marks the message as deprecated. The proto output will reflect this setting.
        If for some reason you want to deprecate the rust struct but not the proto message, you can use `#[deprecated]` along with `#[proto(deprecated = false)]`. Vice versa, you can deprecate the proto message only by using `#[proto(deprecated = true)]`

- `validate`
    - Type: closure or expression, or a list of them surrounded by brackets
    - Example: `#[proto(validate = |v| v.cel(my_cel_rule))]` or `#[proto(validate = [ CustomValidator, *STATIC_VALIDATOR ])]`
    - Description:
        Defines the default validators for the given message. These will be executed inside the message's own [`validate`](crate::ValidatedMessage::validate) method, and whenever the message is used as a field in another message, along with the validators defined for each field. If a closure if used, the default `CelValidator` builder will be passed as the argument, and the validator will be cached in a static Lazy. If another expression is used, it must resolve to an implementor of [`Validator`](crate::Validator) for the message.

- `skip_checks`
    - Type: list of Idents
    - Example: `#[proto(skip_checks(validators))]`
    - Description:
        Disables the generation of tests. Currently, the allowed values are:
            - `validators`: disables the automatic generation of a test that checks the validity of the validators used by the message. The `check_validators_consistency` will still be generated and be available for manual testing.
