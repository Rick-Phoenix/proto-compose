test-renders:
    cargo test -p testing test_renders -- -q --nocapture

[working-directory("testing")]
build-protos:
    cargo run -p testing

build-server:
    cargo build -p test-server

test:
    cargo test --all-features -- -q --nocapture

update-changelog version:
    git cliff --tag {{ version }}
    git add "CHANGELOG.md"
    git commit -m "updated changelog"

release-test version: test
    cargo release {{ version }} -p protoschema

release-exec version: test (update-changelog version)
    cargo release {{ version }} -p protoschema --execute

build-docs:
    RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --all-features --open
