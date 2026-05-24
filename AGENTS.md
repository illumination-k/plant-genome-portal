# AGENTS.md

## Development Servers

Start the API and web development servers together:

```bash
mise r dev
```

This runs `dev:*` tasks:

- `dev:api`: watches Rust API code with `cargo watch` and serves the API on `127.0.0.1:3000`.
- `dev:web`: runs Vite for the web app and proxies API paths to `127.0.0.1:3000`.

Stop both dev servers from another shell:

```bash
mise r stop_dev
```

The dev tasks write PID files under `target/dev/pids`. `stop_dev` reads those files, stops the running processes, and removes stale PID files.
