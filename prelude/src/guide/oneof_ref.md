# Oneof Reference

## Macro attributes

- `proxied`
    - Type: Ident
    - Example: `#[proto_oneof(proxied)]`
    - Description:
        Creates a proxied oneof

## Container attributes:

- `derive`
    - Type: list of Paths
    - Example: `#[proto(derive(Copy))]`
    - Description:
        In case of a proxied impl, it specifies derives to apply to the Proto oneof.
        Shorthand for `#[proto(attr(derive(..)))]`

- `attr`
    - Type: MetaList
    - Example: `#[proto(attr(serde(default)))]`
    - Description:
        Forwards the given metas to the proto oneof.
        In the example above, the proto oneof would receive the attribute as `#[serde(default)]`.

- `options`
    - Type: Expr
    - Example: `#[proto(options = vec![ my_option_1() ])]`
    - Description:
        Specifies the options for the given oneof. It must resolve to an implementor of IntoIterator<Item = [`ProtoOption`](crate::ProtoOption).

- `name`
    - Type: string
    - Example: `#[proto(name = "abc")]`
    - Description:
        Specifies the name of the oneof. Overrides the default behaviour, which uses the snake-case name of the enum.

- `from_proto`
    - Type: function Path or closure
    - Example: `#[proto(from_proto = my_convert_fn)]` or `#[proto(from_proto = |v| v.some_method())]`
    - Description:
        Overrides the automatically generated conversion from proto to proxy.

- `into_proto`
    - Type: function Path or closure
    - Example: `#[proto(into_proto = my_convert_fn)]` or `#[proto(into_proto = |v| v.some_method())]`
    - Description:
        Overrides the automatically generated conversion from proxy to proto.

- `validate`
    - Type: Expr, or bracketed list of Exprs
    - Example: `#[proto(validate = [ CustomValidator, *STATIC_VALIDATOR ])]`
    - Description:
        Defines the default validators for the given oneof. These will be executed inside the oneof's own [`validate`](crate::ValidatedMessage::validate) method, and whenever the oneof in another message, along with the validators defined for each variant. The expressions is it must resolve to an implementors of [`Validator`](crate::Validator) for the oneof.

- `skip_checks`
    - Type: list of Idents
    - Example: `#[proto(skip_checks(validators))]`
    - Description:
        Disables the generation of tests. Currently, the allowed values are:
            - `validators`: disables the automatic generation of a test that checks the validity of the validators used by the message. The `check_validators_consistency` will still be generated and be available for manual testing.

