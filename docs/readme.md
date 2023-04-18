Welcome to Run's documentation. Run is a lightweight, standalone tool that manages tasks and processes for you. It is written 100% in safe Rust, is working on all the major operating systems, and is virtually compatible with any stack or language.

This documentation should give you a good idea of how Run can be benefitial to you, and how you can get the most out of it. If you have questions, feel free to come [discuss](https://github.com/aymericbeaumet/run/discussions).

Here's what "Hello, World!" could look like with Run:

```bash
$ cargo run -- 'echo Hello,' 'printf World!'
[echo] Hello,
[echo] echo Hello, terminated with status code 0
[printf] World!
[printf] printf World! terminated with status code 0
```

You could also use a `run.toml` file, notice how the output is the same:

```bash
$ cat run.toml
[[run]]
cmd = ["echo", "Hello,"]

[[run]]
cmd = ["printf", "World!"]
$ run
[echo] Hello,
[echo] echo Hello, terminated with status code 0
[printf] World!
[printf] printf World! terminated with status code 0
```
