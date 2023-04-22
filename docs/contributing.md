Source code for both the project and documentation lives on
[GitHub](https://github.com/aymericbeaumet/run).

Project roadmap is managed on
[GitHub](https://github.com/users/aymericbeaumet/projects/1).

## How can you help?

- create [an issue](https://github.com/aymericbeaumet/run/issues/new) when you
  face a problem ([search first](https://github.com/aymericbeaumet/run/issues))
- create
  [a discussion](https://github.com/aymericbeaumet/run/discussions/new/choose)
  when you have a question or wants a new feature
- search for
  [opened issues](https://github.com/aymericbeaumet/run/issues?q=is%3Aissue+is%3Aopen)
  that you might want to address
- improve the
  [documentation](https://github.com/aymericbeaumet/run/tree/master/docs)

## Development

The [Rust toolchain](https://www.rust-lang.org/tools/install) has to be
installed to work on this project. It is recommended to install
[watchexec](https://github.com/watchexec/watchexec) as it is very handy to watch
for changes and re-run the code or the tests.

Here are some useful commands:

```bash
cargo run                    # execute `run.toml` with the Run binary
cargo run -- 'echo hello'    # execute a command with the Run binary
cargo run -- --help          # print help
cargo run -- --version       # print version
cargo build                  # build a debug binary in `./target/debug/run`
cargo build --release        # build a release binary in `./target/release/run`
cargo test                   # run tests
cargo fmt                    # format the codebase
cargo fmt --check            # check the codebase format
cargo clippy                 # lint the codebase
cargo upgrade --incompatible # upgrade dependencies (https://crates.io/crates/cargo-edit)
./scripts/release patch      # tag a new release
```
