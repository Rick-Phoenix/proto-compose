# Reusing Oneofs

It is possible to reuse the same rust enum as a oneof for several different messages.

In order for this to happen, the tags for the oneof variants must be defined manually, and they will remain the same for every message that uses them.

When a oneof is used in a message, a completely new oneof will be generated inside that message's definition in its proto file. It will contain the same variants and tags of the original instance, as well as all of the options defined on the original instance. The name will be overwritten and will match the name of the field in the rust struct (unless manually overridden with the `#[proto(name = "..")]` attribute). 

All validators and options defined on the oneof as a field will be local to that instance.

⚠️ Since prost was not originally programmed to support reusable oneofs, it does not check if the tags used with it are wrong and would just produce silent errors at runtime. In order to avoid this, the `#[proto_message]` macro will automatically generate a test that checks if the tags specified on the oneof as a field match the oneof's tags.
