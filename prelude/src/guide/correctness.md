# Correctness

This crate tries to enforce correctness as much as possible at compile/testing time, and it does so in the following ways:

1. Each validator holds a method called [`check_consistency`](crate::Validator::check_consistency) which checks if the inputs inside of it make sense or were caused by an error. All default validators implement this method and perform these checks on all sorts of inputs with a key focus on CEL rules which are written as plain text and are very easy to get wrong. Custom validators can optionally implement this method too.

When using validators in a message or in a oneof, a method for checking all of these validators will be automatically generated and unless indicated otherwise, a test will also be automatically generated, that will call such method and panic on failure.

2. In order to improve debuggability, the package handle will contain a method that checks if there are CEL rules with the same ID within the same message and unless otherwise specified, it will also generate a test that calls such method and panics on failure.

3. Tests are automatically generated for the accuracy of oneof tags (check reusing oneofs section).

