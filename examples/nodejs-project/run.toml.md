```toml
mode = "tmux"

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