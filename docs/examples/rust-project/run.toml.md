> This example is using a _literate Runfile_. [Learn more](../../user-guide/runfile.md#literate-runfiles).

This example shows how you can run your tests and your program in tmux mode.

```toml
mode = "tmux"

[[run]]
cmd = ["cargo", "test"]

[[run]]
cmd = ["cargo", "run"]
```
