[working-directory(".")]
test-all: test-shared-schemas test-schemas
    cargo test -p prelude -- --nocapture

test-schemas:
    cargo test -p testing -- --nocapture
    cargo test -p test-schemas -- --nocapture

test-shared-schemas: gen-schemas
    cargo test -p test-reflection -- --nocapture
    cargo test --features reflection -p test-reflection -- --nocapture

gen-schemas:
    cargo run --bin test-schemas

[working-directory(".")]
expand-reflection: gen-schemas
    cargo expand --features reflection -p test-reflection > expanded.rs

test-renders:
    cargo test -p testing test_renders -- -q --nocapture

update-changelog version:
    git cliff --tag {{ version }}
    git add "CHANGELOG.md"
    git commit -m "updated changelog"

release-test version: test-all
    cargo release {{ version }} -p protoschema

release-exec version: test-all (update-changelog version)
    cargo release {{ version }} -p protoschema --execute

build-docs:
    RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc -p prelude --all-features --open
