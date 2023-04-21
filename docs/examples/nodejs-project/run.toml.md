> This example is using a _literate Runfile_. [Learn more](../../user-guide/runfile.md#literate-runfiles).

This example shows how you can run redis and your node server in tmux mode.

```toml
mode = "parallel"

[[run]]
name = "install"
tags = ["install"]
cmd = ["npm", "install"]

[[run]]
name = "redis"
tags = ["dev"]
cmd = ["docker", "run", "--rm", "-p", "127.0.0.1:6379:6379", "redis:6.2.11-alpine3.17"]

[[run]]
name = "node"
tags = ["dev"]
cmd = ["node", "."]
```
