> This example is using a _literate Runfile_.
> [Learn more](../../user-guide/runfile.md#literate-runfiles).

This run starts three commands in individual panels grouped under a unique tmux window.

```toml
mode = "tmux"

[[run]]
name = "curl"
description = "fetch the current ip address"
cmd = ["curl", "icanhazip.com"]

[[run]]
name = "uname"
cmd = ["uname", "-a"]

[[run]]
name = "df"
cmd = ["df", "-h"]

```
