# Contributing

Thanks for helping improve `sig-impact`.

## Setup

Install [Rust through rustup](https://rustup.rs/) and clone the repository:

```sh
git clone https://github.com/exocognosis/sig-impact.git
cd sig-impact
cargo test
```

The repository's `rust-toolchain.toml` selects the minimum supported toolchain
and installs Rustfmt and Clippy automatically.

## Before submitting a change

Run the same checks used by continuous integration:

```sh
make check
```

If `make` is unavailable, run:

```sh
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo build --release
```

Please add or update tests when changing scheme sizes, arithmetic, parsing, or
output behavior.

## Updating signature sizes

Every size change must include:

1. the exact parameter-set name;
2. an authoritative specification or standards reference;
3. a test asserting the new public-key and signature sizes;
4. matching documentation updates.

Do not silently replace a padded, compressed, or variable-length encoding with
a different encoding profile.

## Pull requests

Keep changes focused. Explain the scenario affected, show a representative
command and output, and confirm that `make check` passes. By participating, you
agree to follow the [Code of Conduct](CODE_OF_CONDUCT.md).
