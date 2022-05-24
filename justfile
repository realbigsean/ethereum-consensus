test:
    cargo test
run-integration-tests:
    cargo test --test '*'
fmt:
    cargo fmt
lint: fmt
    cargo clippy
build:
    cargo build
run-ci: lint build test
gen-spec:
    cargo run --features gen-spec --bin gen-spec
