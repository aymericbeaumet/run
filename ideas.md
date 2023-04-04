Just random ideas as they flow in my brain.

- Allow to add command from cli with `workbench -c '<cmd>'`
- Allow N workbenches
- Allow to filter processes with tags
- Add support for remote workbenches (e.g.: over https)
- Support those options in workbench files:

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
