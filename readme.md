[![GitHub Actions](https://github.com/aymericbeaumet/run/actions/workflows/ci.yml/badge.svg)](https://github.com/aymericbeaumet/run/actions/workflows/ci.yml)

# run

[run](https://github.com/aymericbeaumet/run) is a simple, standalone tool that manages for you the processes you have to run when working on any project.

## Features

- Lightweight: single non-bloated binary
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

_todo_

## Documentation

For more information about the project, including installation, getting started, and many other topics, have look at https://aymericbeaumet.gitbook.io/run/.
