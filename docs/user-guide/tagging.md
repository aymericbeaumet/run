It is common to have different kinds of task in a single Runfile. You can use tags to only run a
specific subset of the tasks. Commands will be executed in the order they are defined.

## Add tags

This is an example of how you can add tags to commands. It is load as the `run.toml` used by the
snippets below.

```toml
[[run]]
cmd = ["echo", "hello"]
tags = ["hello"]

[[run]]
cmd = ["printf", "world"]
tags = ["world"]
```

## Filter by tags

```bash
$ run -t hello
[echo] hello
```

```bash
$ run -t world
[echo] world
```

## Combine several tags

Combining tags act as a logical OR. That is all the commands that matches at least one of the
provided tags will be executed.

You can pass the tag flag several times:

```bash
$ run -t hello -t world
[echo] hello
[printf] world
```

You can also provide a single flag with comma separated flags:

```bash
$ run -t hello,world
[echo] hello
[printf] world
```
