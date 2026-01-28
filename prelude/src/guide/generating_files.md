# Generating Files

When the `inventory` feature is enabled, generating files is done in a single step. You just take the package handle created by the [`proto_package`](crate::proto_package) macro, and call the [`render_files`](crate::package::render_files) method with the desided root directory of the package, and that's it.

# No_std usage

Since the inventory feature relies on the `inventory` crate being available (which is not the case in a no_std crate), collecting the schema items needs one of the following workarounds.

1. You can create a utility crate in your workspace which builds your no_std crate, but enables the `inventory` feature. In this case, the inventory functionalities will work normally, all the items will be picked up automatically like in a std-compatible crate and you will be able to generate the files with a single method call as described above.
This is in theory the simplest way, but there is a big catch. Since rust-analyzer compiles only one version per crate in a workspace (as of today), if you apply this workaround, your IDE will show all sorts of errors, because it will apply the checks to the version of the crate that is using the inventory feature, even if the actual crate is not using the feature. You can check this out by cloning the repo and looking inside the test-no-std crate which is where I tried applying exactly this. If you can find a workaround for this issue, then this should be the way to go.

2. The alternative is to use the [`file_schema`](crate::file_schema) and [`package_schema`](crate::package_schema) macros, which allow you to manually define the elements of a package.

The `file_schema` macro accepts all the inputs of the `define_proto_file` macro, plus the list of messages, enums and services, which are just bracketed lists of paths for each element.
Nested messages and enums are defined by using `ParentMessage = { enums = [ NestedEnum ], messages = [ NestedMsg ] }` instead of just the message's name, as shown below.

The `package_schema` macro simply accepts the name of the package as the first argument, and a bracketed list of idents for the files that it contains.

Example:

```rust
use prelude::*;

#[proto_message]
struct Msg1 {
    id: i32
}

#[proto_message]
#[proto(parent_message = Msg1)]
struct Nested {
    id: i32
}

#[proto_message]
struct Msg2 {
    id: i32
}

#[proto_enum]
enum Enum1 {
    Unspecified, A, B
}

#[proto_enum]
#[proto(parent_message = Msg1)]
enum NestedEnum {
    Unspecified, A, B
}


#[proto_service]
enum MyService {
    GetMsg {
        request: Msg1,
        response: Msg2
    }
}


let manual_file = file_schema!(
    name = "test.proto",
    messages = [
        Msg2,
        Msg1 = { messages = [ Nested ], enums = [ NestedEnum ] }
    ],
    services = [ MyService ],
    enums = [ Enum1 ],
    // Imports, options, etc...
);

let manual_pkg = package_schema!("my_pkg", files = [ manual_file ]);

```


