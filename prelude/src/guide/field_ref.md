# Fields Reference

## Type Attribute

In most cases, the specific protobuf type of the field will be automatically inferred, if primitive types are used.

For special int types (such as sfixed32 or sint32), bytes, oneofs, enums and messages, it may be necessary to specify these manually.

For the scalar types, you can simply use their protobuf name (`sint32`, `bytes`), whereas for oneofs, enums and messages there are a set of special attributes.

- `message`
    - Type: Ident or MetaList
    - Example: `#[proto(message)]` or `#[proto(message(default, proxied))]`
    - Description:
        This can be used just as a plain ident to signal that the target type is an unproxied message. 
        Otherwise, these other attributes can be used inside parentheses:
            - `proxied`: specifies that the message is proxied, so that the proto item will contain the proxied message with the `Proto` suffix and not the proxy itself. Only available for proxied impls.
            - `default`: allows the message to be used without being wrapped in `Option`. In this case, the default conversion from proto for the field will use `Default::default()` if the field is `None` in the proto item. Only available for proxied impls.
            - `boxed`: marks the message as boxed. Automatically inferred in most cases.
            - `(any other path)`: this path will be assumed to be a custom proxy for the given message. Only available for proxied impls.

- `oneof`
    - Type: MetaList
    - Example: `#[proto(oneof(tags(1, 2), default, proxied))]`
    - Description:
        Marks the field as a oneof. Since oneofs are reusable, this will generate a new oneof in the output proto file belonging to the specific message where it's being used, which will have the snake_case name of the field, or a custom name if the `name` attribute is used.

        All the validators being defined on the oneof will be used, and the extra ones used for this specific field will only apply to the containing message.

        All the schema settings of the original oneof such as options will be retained, and the extra ones set on the field will apply only to the specific oneof being generated for this message.

        Unlike other fields, the tags for a given oneof must be set manually, and they must match the tags of the oneof precisely. The macro will automatically generate a test that checks the accuracy of the tags.

        For more information, visit [reusing oneofs].

        Other supported attributes are:
            - `required`: adds a validator for this oneof that checks if a value is set. The `(buf.validate.oneof).required` option will also be added to the schema of the oneof (only for the specific instance of the oneof, not is other places where it's being reused unless specified).
            - `proxied`: specifies that the oneof is proxied, so that the proto message will contain the proxied oneof with the `Proto` suffix and not the proxy itself. Only available for proxied impls.
            - `default`: allows the oneof to be used without being wrapped in `Option`. In this case, the default conversion from proto for the field will use `Default::default()` if the field is `None` in the proto item. `Default` should be implemented for the target oneof. Only available for proxied impls.
            - `(any other path)`: this path will be assumed to be a custom proxy for the given oneof. Only available for proxied impls.

- `enum_`
    - Type: Ident or MetaList
    - Example: `#[proto(enum_)]` or `#[proto(enum_(MyEnum))]`
    - Description:
        Specifies that the field is a proto enum. In proxied impls, the path to the specific enum should be inferred automatically, but in direct impls, the path should be specified inside parentheses.

### Cardinality Attributes

The cardinality of the field should be inferred automatically in most cases from the given type (BTreeMap/HashMap -> map, Vec -> repeated, Option -> optional (except for messages)), but if it's not, then it can be specified as follows:

- `repeated` -> `repeated(target_type_as_above)`
- `optional` -> `optional(target_type_as_above)`
- `map` -> `map(key_type_as_above, value_type_as_above)`

## Other Attributes

- `ignore`
    - Type: Ident
    - Example: `#[proto(ignore)]`
    - Description:
        Excludes the field from the proto item (only valid in proxied impls).

- `attr`
    - Type: list of Metas
    - Example: `#[proto(attr(default))]`
    - Description:
        Forwards the given metas to the proto field.
        In the example above, the target field would receive the attribute as `#[default]`.

- `options`
    - Type: Expr
    - Example: `#[proto(options = vec![ my_option_1() ])]`
    - Description:
        Specifies the options for the given field. It must resolve to an implementor of IntoIterator<Item = [`ProtoOption`](crate::ProtoOption).

- `name`
    - Type: string
    - Example: `#[proto(name = "abc")]`
    - Description:
        Specifies the name of the field. Overrides the default behaviour, which uses the snake_case name of the field.

- `from_proto`
    - Type: function Path or closure
    - Example: `#[proto(from_proto = my_convert_fn)]` or `#[proto(from_proto = |v| OtherType { val: v.val })]`
    - Description:
        Overrides the automatically generated conversion from proto to proxy. If the field is marked with `ignore`, a closure cannot be used.

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
        If for some reason you want to deprecate the rust field but not the proto field, you can use `#[deprecated]` along with `#[proto(deprecated = false)]`. Vice versa, you can deprecate the proto field only by using `#[proto(deprecated = true)]`

- `validate`
    - Type: closure or expression, or a list of them surrounded by brackets
    - Example: `#[proto(validate = |v| v.cel(my_cel_rule))]` or `#[proto(validate = [ CustomValidator, *STATIC_VALIDATOR ])]`
    - Description:
        Defines the validators for the given field. These will be executed inside the container's own [`validate`](crate::ValidatedMessage::validate) method. If a closure if used, the default validator builder for the given type will be passed as the argument, and the validator will be cached in a static Lazy. If another expression is used, it must resolve to an implementor of [`Validator`](crate::Validator) for the target type.


- `tag`
    - Type: number
    - Example: `#[proto(tag = 1)]`
    - Description:
        Sets the protobuf tag for the given field. Tags are mandatory for oneof variants, but not for messages, where they can be automatically generated.
