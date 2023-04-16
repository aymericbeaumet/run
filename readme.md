[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/aymericbeaumet/run/ci.yml?branch=master&logo=github)](https://github.com/aymericbeaumet/run/actions/workflows/ci.yml) [![GitHub release (latest by date)](https://img.shields.io/github/v/release/aymericbeaumet/run)](https://github.com/aymericbeaumet/run/releases) [![GitHub](https://img.shields.io/github/license/aymericbeaumet/run)](https://github.com/aymericbeaumet/run-cli/blob/master/license.md)

# run

run is a simple, standalone tool that manages your processes for any project.

Learn more at the [run](https://www.run-cli.org) website.

## Features

- Lightweight: single non-bloated binary
- Convenient: native tmux integration for unmatched productivity
- Portable: works on Linux, macOS and Windows

## Quickstart

```bash
# go
run -m tmux --watch 'go run .'

# node.js
run -m tmux --watch 'docker-compose up' 'npm run dev'

# rust
run -m tmux --watch 'cargo test' 'cargo run'
```

For more examples, have a look at the [examples directory](./examples).

## Install

```bash
# todo
```

Visit [the install documentation](https://www.run-cli.org/installation) for an exhaustive list of the installation methods.

## Documentation

For more information about the project, including installation, getting started, and many other topics, have look at the [run](https://www.run-cli.org) website.
