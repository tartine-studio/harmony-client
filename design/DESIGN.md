# Harmony client — design reference

Mockups for the desktop client, made on a design canvas. Use them as the visual reference when building screens in `src/`.

## Files

| File | Screen |
|---|---|
| `mockups/Login.dc.html` | Sign in / sign up |
| `mockups/Main.dc.html` | App shell: channel sidebar, text channel, member list. Also renders the voice channel view (pick a voice channel) |
| `mockups/Voice.dc.html` | Same shell, opened on a voice channel (imports `Main`) |
| `mockups/Mobile.dc.html` | Narrow-width text channel (390×844), reference for small windows |

### How to read a `.dc.html` file

These are design-canvas sources, not app code, and they don't run on their own in a browser.

- The markup inside `<x-dc>` is the layout. Styles are inline.
- `{{name}}` holes are filled by `renderVals()` in the `<script type="text/x-dc">` block at the bottom. That script holds the sample data (users, channels, messages) and the UI state logic (selected channel, mute, deafen, invisible…).
- `<sc-for>` is a loop and `<sc-if>` a conditional.

Take layout, spacing, sizes, copy and behavior from them. Never copy the raw hex colors into components: map them to the theme variables below.

## Visual direction

Dark graphite surfaces with a sage-green accent. Only a dark theme was designed; the light theme needs its own pass.

### Mapping to `src/lib/styles/theme.css` (dark)

| Variable | Mockup value | Used for |
|---|---|---|
| `--color-bg` | `#1B1D21` | Main content area (messages, voice tiles area) |
| `--color-bg-elevated` | `#262A2F` | Message composer, active channel row (`#2A2E34`) |
| `--color-surface` | `#23272C` | Voice tiles, popovers, control bar |
| `--color-text` | `#ECEDEE` | Primary text |
| `--color-text-muted` | `#A4AAB2` | Secondary text, inactive channel names |
| `--color-accent` | `#7CD4A8` | Logo, primary buttons, speaking ring, toggles |
| `--color-accent-contrast` | `#0F2A1E` | Text and icons on the accent |
| `--color-border` | `#24272C` | Dividers between panels |
| `--color-status-online` | `#4FCB8C` | Presence dot |
| `--color-status-idle` | `#F0B44C` | Presence dot |
| `--color-status-offline` | `#7A8189` | Hollow presence ring (also "invisible", shown to self) |
| `--color-danger` | `#E5665A` | "Leave" button background (text `#1A0A08` on it) |

### New variables the mockups need

| Variable | Value | Used for |
|---|---|---|
| `--color-bg-sunken` | `#17191C` | Channel sidebar and member list |
| `--color-bg-deep` | `#141619` | User panel at the bottom of the sidebar |
| `--color-text-subtle` | `#8A919A` | Section labels, timestamps, channel icons |
| `--color-border-strong` | `#30353B` | Input and popover borders |
| `--color-danger-soft` | `#2A1F1E` | Background of active mute/deafen buttons |
| `--color-danger-text` | `#F59A8C` | Muted/deafened icons |
| `--color-danger-contrast` | `#1A0A08` | Text on `--color-danger` |

All text pairs above meet WCAG AA (4.5:1) on their surface.

### Type

- UI: **Instrument Sans** (400–700). Body 15px, channel header 16px/600, section labels 12px/600 uppercase with 0.06em tracking.
- Timestamps and server address: **JetBrains Mono** 11–14px.
- Bundle both fonts in `static/` rather than loading Google Fonts, so the app works offline. Then set `--font-sans` and `--font-mono`.

### Shape and spacing

- Radius: 6–8px (rows, icon buttons), 10–12px (inputs, popovers), 14–16px (voice tiles), full (avatars, pills).
- Sidebar 272px, member list 240px, header height 56px.
- Avatars: initials on a per-user tint (the API has no avatar images). Sizes 24 (voice list), 32 (members), 36 (user panel), 40 (messages), 88 (voice tiles).
- Icons: 2px stroke line icons, 15–22px.

## Screens and server API

| UI | Server |
|---|---|
| Login | `POST /api/auth/login` `{email, password}` → `{accessToken, refreshToken}`. The "server address" field is the base URL of the instance, stored client-side |
| Sign up | `POST /api/auth/register` `{username (2–32), email, password (8–128)}` |
| Token refresh | `POST /api/auth/refresh` `{refreshToken}` |
| Channel sidebar | `GET /api/channels` (type `text` or `voice`). "+" buttons → `POST /api/channels` `{name, type}` |
| Message list | `GET /api/channels/{id}/messages`. Live updates over WebSocket: `messageCreated`, `messageUpdated`, `messageDeleted` |
| Composer | WS `sendMessage` (content 1–2000 chars) |
| Edit / delete own message | WS `editMessage`, `deleteMessage` |
| Member list | `GET /api/users` and presence over WS: `presenceSnapshot`, `presenceBroadcast` |
| Status popover | Status is automatic (online/idle derived from activity, send WS `activity`). The only user choice is "appear offline" → `invisible` (`PATCH /api/users/me` / WS `presenceUpdate`) |
| Voice channel join / leave | WS `joinVoice`, `leaveVoice`, then WebRTC: `voiceOffer` → `voiceAnswer`, `voiceCandidate` |
| Mute / deafen | WS `updateVoiceState` (`selfMuted`, `deafened`). Deafened implies muted |
| Who is in voice, speaking ring | `voiceStateSnapshot`, `voiceStateUpdate`, `voiceSpeaking` |
| Voice disconnected (not yet designed) | `voiceDisconnected` with reason `otherDevice`, `channelDeleted` or `connectionFailed` |

WebSocket: `GET /api/ws?token=<accessToken>`.

## Not designed yet

- Light theme values
- Banner for `voiceDisconnected`
- Create / rename / delete channel dialogs
- Message edit mode, settings screen, error and loading states
