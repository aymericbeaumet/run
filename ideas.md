Just random ideas as they flow in my brain.

- Allow to add an inline command from cli with `workbench -c '<cmd>'`
- Allow to pass N workbench files as cli args
- Allow to filter processes with tags
- Add support for remote workbenches (e.g.: over https)
- Detect http://....:... patterns in stdout, and add them to panel title in tmux
- Add a "ready" condition to config to allow waiting for a specific state (e.g.: http//.../health returns 200)
- Allow to pass the mode as a cli flag

Maybe support those options in workbench files?

```
# load_dotenv = true
# root = ../www
# environment = { NODE_ENV = "production" }

#[[init]]
#cmd = ""

#[[test]]
#cmd = ""

#[[dev]]
#cmd = ""

[[run]]
cmd = "ls"
#watch = true

[[run]]
cmd = ["ls"]
#once = true
#tags = ["test"]
#after = ["tag1"]
#before = ["tag2"]
```
