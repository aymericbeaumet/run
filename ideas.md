- Add support for remote runs (e.g.: over https)
- Detect http://....:... patterns in stdout, and add them to panel title in tmux
- Add a "wait_for" condition to config to allow waiting for a specific state (e.g.: http//.../health returns 200)
- Allow to use the `run #install #dev` shortcut instead of `run -t install -t dev`
- Allow to configure options for specific tags
- Automatic support for node_modules/.bin
- Support env expansion in commands
- Add a --dry-run flag that basically prints the commands instead of running
- Add null-ls support in neovim (run check?)
- Scope the config at the global/file/run level (cli/env = global)
- tmux: report status code in the pane title
- tmux: report time to finish in the pane title
- tmux: use `run -f /dev/null 'command' inside each panel (prefix, openai, watcher, etc)`
- Add support for secret management (vault, aws secrets, etc)
- Add support for micro-services discovery (k8s, consul, docker, etc)
- Add a literate mode that supports loading .toml.md
- Output dependency graph in dot format
- Release checksum
- Homebrew install
- Add native support for a local .bin directory

[production-ready]

- Add systemd integration/support/compatibility
- Add reverse-proxy support for 0 downtime
