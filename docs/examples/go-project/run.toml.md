> This example is using a _literate Runfile_. [Learn more](../../user-guide/runfile.md#literate-runfiles).

This example shows how you can run your Go tests and program in tmux mode.

```toml
mode = "parallel"

[[run]]
cmd = ["go", "test", "./..."]

[[run]]
cmd = ["go", "run", "."]
```
