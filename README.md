# pumpkin-tabtps

A Pumpkin plugin that displays live server performance stats (TPS and MSPT) in the Minecraft player's tab list footer.

## Overview

This plugin is a Pumpkin-based implementation inspired by [jpenilla/TabTPS](https://github.com/jpenilla/TabTPS), bringing similar tab list performance metrics (TPS/MSPT) to the [Pumpkin](https://github.com/Pumpkin-MC/Pumpkin) server framework.

`pumpkin-tabtps` is a plugin for the Pumpkin Minecraft server framework. It updates the tab list footer every second with real-time TPS (ticks per second) and MSPT (milliseconds per tick) readings, color-coded based on server performance.

## Status

This plugin is an early port of the upstream Java TabTPS to Pumpkin's WASM
plugin runtime. Only a small slice of the upstream feature surface is in place
today — see [Feature comparison](#feature-comparison-with-upstream-tabtps) and
the [Roadmap](#roadmap) for what is planned next.

### Implemented (v0.1.0)

- [x] Tab list **footer** display with live TPS + MSPT
- [x] MSPT-based color coding (green ≤ 25, gold ≤ 40, red > 40)
- [x] 1-second refresh via `scheduler::schedule_repeating_task`
- [x] Per-player task lifecycle — scheduled on `PlayerJoinEvent`, cancelled when the player disconnects
- [x] Built as a `wasm32-wasip2` component, drop-in to `pumpkin/plugins/`

### Example Output

![image](https://github.com/user-attachments/assets/4a895184-32df-4f54-b55e-b8b5bb95d65d)

Colors:
- 🟢 **Green**: MSPT ≤ 25
- 🟠 **Gold**: MSPT > 25 and ≤ 40
- 🔴 **Red**: MSPT > 40

## Feature comparison with upstream TabTPS

Legend: ✅ done · ⚠️ partial · ❌ missing · ⏸ deferred

| Category   | Feature                                  | Upstream | pumpkin-tabtps     |
|------------|------------------------------------------|:--------:|:------------------:|
| Display    | Tab list (header + footer)               | ✅       | ⚠️ footer only      |
| Display    | Action bar                               | ✅       | ❌                  |
| Display    | Boss bar (with progress)                 | ✅       | ❌                  |
| Module     | TPS (current)                            | ✅       | ✅                  |
| Module     | MSPT (current)                           | ✅       | ✅                  |
| Module     | TPS rolling averages (1 m / 5 m / 15 m)  | ✅       | ❌                  |
| Module     | Player count                             | ✅       | ❌                  |
| Module     | Ping                                     | ✅       | ❌                  |
| Module     | Memory                                   | ✅       | ❌                  |
| Module     | CPU                                      | ✅       | ❌                  |
| Commands   | `/tabtps toggle <tab\|actionbar\|bossbar>` | ✅     | ❌                  |
| Commands   | `/tabtps reload`                         | ✅       | ❌                  |
| Commands   | `/tickinfo` (alias `/mspt`)              | ✅       | ❌                  |
| Commands   | `/memory` (`/mem`, `/ram`)               | ✅       | ❌                  |
| Commands   | `/ping`, `/pingall`                      | ✅       | ❌                  |
| Config     | `main.conf` (HOCON)                      | ✅       | ❌                  |
| Config     | `display-configs/` per-permission        | ✅       | ❌                  |
| Config     | `themes/` (color sets, gradient)         | ✅       | ❌                  |
| Other      | i18n (multi-locale messages)             | ✅       | ❌                  |
| Other      | Per-player display toggle (persisted)    | ✅       | ❌                  |
| Other      | Update checker                           | ✅       | ⏸ network perm.    |

Most upstream behaviour maps cleanly onto Pumpkin's plugin API
(`server.get_tps`, `server.get_mspt`, `server.get_player_count`,
`server.get_sys_info`, `player.get_ping`, `player.get_locale`, the `boss-bar`
resource, `show-actionbar`, the command builder, the `i18n` interface, and
`context.get_data_folder`). The remaining work is reimplementation in Rust
rather than waiting on host capabilities.

## Roadmap

Each phase is intended to be small enough to land on its own. Earlier phases
unblock later ones (modular display → config → commands → theming).

### Phase 1 — Modular display foundation

- [ ] Extract the footer renderer into a `Module` trait (TPS / MSPT first) so additional modules can drop in.
- [ ] Render the tab list **header** in addition to the footer.
- [ ] **Player Count** module (`server.get_player_count` / `server.get_max_players`).
- [ ] **Ping** module (`player.get_ping`).

### Phase 2 — Configuration

- [ ] Persist a config file under `context.get_data_folder()` (TOML, e.g. `tabtps.toml`).
- [ ] Make color thresholds, update interval, and the active module list configurable.
- [ ] Declare `fs.read.data` / `fs.write.data` in `PluginMetadata::permissions`.

### Phase 3 — Additional display targets

- [ ] **Action bar** display via `player.show_actionbar`.
- [ ] **Boss bar** display via the `boss-bar` resource — title + 0.0–1.0 progress mapped to TPS or MSPT, with color shifting on thresholds.
- [ ] Per-player toggle state (in-memory first, persisted in Phase 5).

### Phase 4 — Rolling averages

- [ ] In-plugin TPS sampler (1 m / 5 m / 15 m windows) polling `get_tps` on a fixed cadence — upstream computes these itself, and Pumpkin's API only exposes a single current value.
- [ ] Surface the rolling windows in the TPS / MSPT modules and the `/tickinfo` command.

### Phase 5 — Commands

- [ ] `/tabtps toggle <tab|actionbar|bossbar>` — per-player display toggle.
- [ ] `/tabtps reload` — reload the config.
- [ ] `/tickinfo` (alias `/mspt`) — TPS + MSPT averages.
- [ ] `/ping`, `/ping <name>`, `/pingall`.
- [ ] `/memory` / `/mem` / `/ram` — uses `server.get_sys_info()` (requires `sys.info.ram`).
- [ ] Register the matching permission nodes (`tabtps.command.*`, `tabtps.toggle.*`).

### Phase 6 — Theming & i18n

- [ ] Theme support: configurable color sets and gradient (port of upstream `Gradient`).
- [ ] Wire the host `i18n` interface — `load_translations` at startup, `translate` keyed by `player.get_locale()`.
- [ ] Ship the same baseline keys upstream uses (`messages.properties`).

### Phase 7 — Polish

- [ ] **CPU module** via `server.get_sys_info()` (`sys.info.cpu`). Upstream tracks process vs. system CPU with a 500 ms sampler — Pumpkin only exposes a snapshot, so this module will be coarser.
- [ ] Per-permission "display configs" (`tabtps.defaultdisplay`, …) à la upstream.
- [ ] Update checker — **deferred**: requires `network.outbound`, a heavy permission to ask of server operators. Revisit if there is demand.

## Project Structure

```
pumpkin-tabtps
├── Cargo.toml          # Rust package manifest
├── LICENSE             # MIT License
└── src
    ├── lib.rs          # Plugin entry point (Plugin trait impl + register_plugin!)
    └── join_handler.rs # PlayerJoinEvent handler + tab footer update task
```

## Usage

1. **Build the Plugin**
    ```bash
    cargo build --release --target wasm32-wasip2
    ```

2. **Deploy**

    Copy the resulting `tabtps.wasm` file from `target/wasm32-wasip2/release/`
    to your Pumpkin server's `plugins/` directory.

3. **Run Server**

    Start your Pumpkin server, and the plugin will:

    * Log `Hello, TabTPS!` on load
    * Start updating the tab footer every second when a player joins

## Dependencies

* [`pumpkin-plugin-api`](https://github.com/Pumpkin-MC/Pumpkin) (git, `master`) — provides the `Plugin` trait, `Context`, `Server`, scheduler, event handlers, command builder, boss bar, i18n, and `TextComponent`. Upstream API is unstable; expect to chase breaking changes when bumping.
* [`tracing`](https://docs.rs/tracing/) — structured logging via the host server.

## License

This project is licensed under the MIT License. See [`LICENSE`](./LICENSE) for details.
