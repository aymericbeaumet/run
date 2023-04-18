Run supports literate Runfiles with the extension `.toml.md`. This allows to embed TOML code blocks within classic markdown code.
This is convenient to explain workflows, and is working well with generated documentation.

All the TOML code blocks would be concatenated as is before being parsed as one.

For example, the following `run.toml.md` declares two commands:

````markdown
# Development

This Runfile takes care of centralizing all the commands needed to run our stack.
A single `run -f run.toml.md` is enough to start all the services.

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
