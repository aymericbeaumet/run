Modes allow to specify the way commands should be executed by Run. There are three modes: sequential, parallel, and tmux. The default is sequential.

## Sequential

The commands are executed sequentially. Any command running will block the following commands execution. This is the default mode and should be omitted from your Runfile or the CLI arguments.

```bash
# from the CLI
$ run -m sequential
```

```toml
# in your run.toml
mode = "sequential"
```

## Parallel

The commands are executed in parallel. No command execution will be blocked.

```bash
# from the CLI
$ run -m parallel
```

```toml
# in your run.toml
mode = "parallel"
```

## Tmux

The commands are executed in individual tmux panes stacked vertically. No command execution will be blocked.

```bash
# from the CLI
$ run -m tmux
```

```toml
# in your run.toml
mode = "tmux"
```
