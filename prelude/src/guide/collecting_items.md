# Collecting Items

When using the `inventory` feature, most of the schema functionalities will occur automatically, but there are still a few actions that need to be done to declare a proto package and its files inside a rust crate.

## Creating A Package

Start by creating a package using the [`proto_package`](crate::proto_package) macro.

The first parameter of the macro is the ident that will be used for the generated constant that will hold the package handle, which will be used to generate the package and its proto files.

The other parameters are not positional and are as follows:

- `name` (required)
    Type: string
    Example: `proto_package!(MY_PKG, name = "my_pkg")`
    Description:
        The name of the package.


- `no_cel_test`
    Type: Ident
    Example: `proto_package!(MY_PKG, name = "my_pkg", no_cel_test)`
    Description:
        By default, the macro will automatically generate a test that will check for collisions of CEL rules with the same ID within the same message. You can use this ident to disable this behaviour. The [`check_unique_cel_rules`](crate::Package::check_unique_cel_rules) method will still be available if you want to call it manually inside a test.

You can then use the specified ident to refer to the package to generate the files or to link to other files.

## Creating Files

A file must always be in scope for any given item. A new file can be defined with the [`define_proto_file`](crate::define_proto_file) macro.

The first argument is the ident that will be used to refer to the file handle. 
The other parameters are not positional and are as follows:

- `name` (required)
    Type: string
    Example: `define_proto_file!(MY_FILE, name = "my_file.proto", package = MY_PKG)`
    Description:
        The name of the file.

- `package` (required)
    Type: Ident
    Example: `define_proto_file!(MY_FILE, name = "my_file.proto", package = MY_PKG)`
    Description:
        The ident of the package handle.


- `extern_path`
    Type: string
    Example: `define_proto_file!(MY_FILE, name = "my_file.proto", package = MY_PKG, extern_path = "module::path")`
    Description:
        The rust path to reach the items described in this proto file, when applied from an external crate. The items in this file will inherit the path of their file + their own ident. For example, if a message `Msg1` is assigned to this file, its `extern_path` will be registered as `::module::path::Msg1`. 
        It defaults to `core::module_path!()` and should only be overridden for re-exported items where their path does not match their module's path.

- `options`
    - Type: Expr
    - Example: `define_proto_file!(MY_FILE, name = "my_file.proto", package = MY_PKG, options = vec![ my_option() ])`
    - Description:
        Specifies the options for the given file. It must resolve to an implementor of IntoIterator<Item = [`ProtoOption`](crate::ProtoOption).


- `imports`
    - Type: Expr
    - Example: `define_proto_file!(MY_FILE, name = "my_file.proto", package = MY_PKG, imports = vec![ "import1", "import2" ])`
    - Description:
        Specifies the imports for the given file. In most occasions, the necessary imports will be added automatically so this should only be used as a fallback mechanism. It should resolve to an implementor of `IntoIterator` with the items being either `String`, `Arc<str>`, `Box<str>` or `&'static str`.


- `extensions`
    - Type: bracketed list of Paths
    - Example: `define_proto_file!(MY_FILE, name = "my_file.proto", package = MY_PKG, extensions = [ MyExtension ])`
    - Description:
        Specifies the extensions for the given file. The items inside the list should be structs marked with the `#[proto_extension]` macro or implementors of [`ProtoExtension`](crate::ProtoExtension).


- `edition`
    - Type: [`Edition`](crate::Edition)
    - Example: `define_proto_file!(MY_FILE, name = "my_file.proto", package = MY_PKG, edition = Proto3)`
    - Description:
        A value from the [`Edition`](crate::Edition) enum. Supports editions from Proto3 onwards.

### Reusing A File

If you want to define items in different rust files that are descendants of the same module, and place them into the same proto file, you can use one of two macros to bring the file into scope.

- The [`use_proto_file`](crate::use_proto_file) macro, which brings the file into scope and applies the extern path of its own `module_path!()` output
- The [`inherit_proto_file`](crate::inherit_proto_file) macro, which does the same but keeps the import path of the parent module (for re-exported items).

```rust
use prelude::*;

proto_package!(MY_PKG, name = "my_pkg");
define_proto_file!(MY_FILE, name = "my_file.proto", package = MY_PKG);

pub mod submod {
    use super::MY_FILE;
    
    // The file is now in scope, and will be picked up automatically by all items defined in this module
    use_proto_file!(MY_FILE);

    // This message will have the extern path of the `module_path!()` output in here, so `::cratename::submod`
    #[proto_message]
    pub struct Msg {
       pub id: i32
    }
}

pub use re_exported::Msg;
mod re_exported {
    use super::MY_FILE;

    // The file is now in scope, and will be picked up automatically by all items defined in this module
    inherit_proto_file!(MY_FILE);

    // This message will have the extern path of the parent module
    #[proto_message]
    pub struct Msg {
        pub id: i32
    }
}

```

⚠️ Warning

Under the hood, the file macros generate a constant named `__PROTO_FILE` that is picked up by the macro output for each item. This constant is private to the module and hidden so that it cannot be brought into scope accidentally, but using global import from children modules like `use super::*` will bring it into scope. It's not recommended to rely on this method to bring the file into scope and to use the [`use_proto_file`](crate::use_proto_file) macro for more clarity.

In a case where you need to use a global import from the parent module but you want the items to be in a separate proto file, then you must make sure to define a new file with the [`define_proto_file`](crate::define_proto_file) macro, or the items will be picked up by the wrong file.
