## Runfiles

Runfiles are `.toml` files that allow to describe commands to be run. The
default path being looked up by the CLI is `run.toml`.

### Example `run.toml`

```toml
[[run]]
workdir = "./api"
cmd = ["node", "."]

[[run]]
workdir = "./www"
cmd = ["npm", "run", "dev"]
```

## Literate Runfiles

Run supports literate Runfiles with the extension `.toml.md`. This allows to
embed TOML code blocks within markdown documents. This is convenient to document
workflows, and is working well with generated docs.

You have to use the fence notation and specify the _toml_ language: ` ```toml `.
The indented notation is not allowed.

All the TOML code blocks are concatenated, and are then being parsed as a single
entity.

### Example `run.toml.md`

````markdown
# Development

This Runfile takes care of centralizing all the commands needed to run our
stack. A single `run -f run.toml.md` is enough to start all the services.

## Start the backend

```toml
[[run]]
workdir = "./api"
cmd = ["node", "."]
```

## Start the frontend

```toml
[[run]]
workdir = "./www"
cmd = ["npm", "run", "dev"]
```
````
