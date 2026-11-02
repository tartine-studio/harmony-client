# Harmony Client

**Harmony** is an open-source, self-hostable alternative to Discord. This repository is the **official desktop client**, built with [Tauri 2](https://tauri.app) (Rust) and [SvelteKit](https://svelte.dev) (Svelte 5, TypeScript).

The server lives at [tartine-studio/harmony-server](https://github.com/tartine-studio/harmony-server), which also documents the client protocol.

## Project status

Early development. Working today: sign up / log in to a Harmony server (session kept in the OS keychain), text channels with live messages (send, edit, delete), member list with presence and "appear offline". Voice channels are listed but not joinable yet. UI in French and English.

## Development

Requirements:

- [Rust](https://rustup.rs) (stable)
- Node.js 22+ and [pnpm](https://pnpm.io)
- The [Tauri system dependencies](https://tauri.app/start/prerequisites/) for your OS

```bash
pnpm install
pnpm tauri dev      # run the desktop app with hot reload
pnpm check          # type-check the frontend
pnpm test           # frontend unit tests (vitest)
pnpm bindings       # regenerate src/lib/bindings from the Rust types
pnpm tauri build    # build a release bundle

cd src-tauri && cargo test     # Rust unit tests
# End-to-end test of the Rust core against a running server with a text channel:
HARMONY_E2E_URL=http://127.0.0.1:8080 cargo test e2e -- --ignored
```

## Structure

```
src/                  # Frontend (SvelteKit, SPA mode via adapter-static): UI only
  routes/             # Pages: boot, /login, /app/channels/[id]
  lib/core/           # Typed bridge to the Rust commands and events
  lib/bindings/       # TS types generated from Rust (ts-rs), don't edit
  lib/state/          # View state (Svelte runes)
  lib/chat/           # Pure UI logic (grouping, presence, message list), unit tested
  lib/components/     # UI components
  lib/i18n/           # fr (reference) and en strings
  lib/styles/         # Global styles and theme tokens
src-tauri/            # Rust core (Tauri)
  src/api/            # REST client (reqwest), token refresh
  src/gateway/        # WebSocket (tokio-tungstenite): reconnection, request/broadcast matching
  src/session.rs      # Login, register, restore, logout; refresh token in the OS keychain
  src/commands.rs     # Tauri commands exposed to the frontend
  tauri.conf.json     # App configuration
  capabilities/       # Permissions granted to the frontend
static/               # Static assets
```

## Architecture

All networking and protocol logic lives in Rust: REST calls, the WebSocket (authenticated with an `Authorization` header), token refresh, reconnection and request correlation. Tokens never reach the webview. The frontend calls Tauri commands and listens to three events: `gateway:event`, `gateway:status` and `session:expired`.

## Theming

Every color comes from semantic CSS variables in [`src/lib/styles/theme.css`](src/lib/styles/theme.css) (`--color-bg`, `--color-text`, `--color-accent`, …). A theme is one set of values for those variables, selected with the `data-theme` attribute on `<html>`. When no theme is set, the app follows the OS light/dark preference. Components must never use raw colors.

## License

[AGPL-3.0](LICENSE)
