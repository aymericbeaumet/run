Modes allow to specify the way commands should be executed by Run. There are three modes: sequential, parallel, and tmux. The default is sequential.

## Sequential

```bash
$ run -m sequential
```

```toml
# run.toml
mode = "sequential"
```

The commands are executed sequentially. Any command running will block the following commands execution. This is the default mode and should be omitted from your Runfile or the CLI arguments

## Parallel

```bash
$ run -m parallel
```

```toml
# run.toml
mode = "parallel"
```

The commands are executed in parallel. No command execution will be blocked.

## Tmux

```bash
$ run -m tmux
```

```toml
# run.toml
mode = "tmux"
```

The commands are executed in individual tmux panes. No command execution will be blocked.
