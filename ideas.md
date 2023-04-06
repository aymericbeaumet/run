Just random ideas as they flow in my brain.

- Allow to pass N workbench files as cli args
- Add support for remote workbenches (e.g.: over https)
- Detect http://....:... patterns in stdout, and add them to panel title in tmux
- Add a "wait_for" condition to config to allow waiting for a specific state (e.g.: http//.../health returns 200)
- Allow to use the `workbench #install #dev` shortcut instead of `workbench -t install -t dev`
- Allow to configure options for specific tags
- Rename the project `run`
- Automatic support for node_modules/.bin
- Support env expansion in commands
- Offer to send stderr to chatgpt to get suggestions
- Test --workdir cli flag (require .toml.args support in tests)
- Add a --dry-run flag that basically prints the commands instead of running
- Add a --dump flag that prints the whole config instead of running

Maybe support those options in workbench files?

```
# load_dotenv = true
# root = ../www
# environment = { NODE_ENV = "production" }

[init]]
cmd = ""
watch = true
include = []
exclude = []
clear = true

[[test]]
cmd = ""

[[dev]]
cmd = ""

[[run]]
cmd = "ls"
watch = true

[[run]]
cmd = ["ls"]
once = true
tags = ["test"]
after = ["tag1"]
before = ["tag2"]
```
