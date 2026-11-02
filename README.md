# Harmony Client

**Harmony** is an open-source, self-hostable alternative to Discord. This repository is the **official desktop client**, built with [Tauri 2](https://tauri.app) (Rust) and [SvelteKit](https://svelte.dev) (Svelte 5, TypeScript).

The server lives at [tartine-studio/harmony-server](https://github.com/tartine-studio/harmony-server), which also documents the client protocol.

## Project status

Early development: the app is a bare shell and doesn't connect to a server yet.

## Development

Requirements:

- [Rust](https://rustup.rs) (stable)
- Node.js 22+ and [pnpm](https://pnpm.io)
- The [Tauri system dependencies](https://tauri.app/start/prerequisites/) for your OS

```bash
pnpm install
pnpm tauri dev      # run the desktop app with hot reload
pnpm check          # type-check the frontend
pnpm tauri build    # build a release bundle
```

## Structure

```
src/                  # Frontend (SvelteKit, SPA mode via adapter-static)
  routes/             # Pages
  lib/styles/         # Global styles and theme tokens
src-tauri/            # Rust backend (Tauri)
  src/                # Rust code
  tauri.conf.json     # App configuration
  capabilities/       # Permissions granted to the frontend
static/               # Static assets
```

## Theming

Every color comes from semantic CSS variables in [`src/lib/styles/theme.css`](src/lib/styles/theme.css) (`--color-bg`, `--color-text`, `--color-accent`, …). A theme is one set of values for those variables, selected with the `data-theme` attribute on `<html>`. When no theme is set, the app follows the OS light/dark preference. Components must never use raw colors.

## License

[AGPL-3.0](LICENSE)
