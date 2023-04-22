Run is a lightweight, standalone tool that manages tasks and processes for you. It is entirely
written in safe Rust, is working on all the major operating systems, and is virtually compatible
with any stack or language.

This documentation should give you a good idea of how Run can be benefitial to you, and how you can
get the most out of it. If you have questions, feel free to come
[discuss](https://github.com/aymericbeaumet/run/discussions).

## Hello, World!

Here's what the infamous _Hello, World!_ could look like with Run:

```bash
$ run 'echo Hello,' 'printf World!'
[echo] Hello,
[printf] World!
```

You could also use a Runfile to achieve the same result:

```toml
# run.toml

[[run]]
cmd = ["echo", "Hello,"]

[[run]]
cmd = ["printf", "World!"]
```

```bash
$ run
[echo] Hello,
[printf] World!
```

This behavior is deterministic as Run executes the commands sequentially by default.

> You will find if you run these examples that Run also prints the exit code on the standard error.
> For brevity it has been omitted in the samples above.

## Next steps

Have a look at the [Installation](./installation.md), read the
[Getting Started](./getting-started/first-steps.md) or dive into the
[User Guide](./user-guide/introduction.md).
