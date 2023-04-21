It is common to have different kinds of task in a single Runfile.
You can use tags to only run a specific subset of the tasks.

## Add tags

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
