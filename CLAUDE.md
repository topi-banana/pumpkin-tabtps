# pumpkin-tabtps

Pumpkin (Minecraft server framework) plugin that displays live TPS / MSPT in the
player tab list footer. Built as a `wasm32-wasip2` component and dropped into
`pumpkin/plugins/`.

## Build

```bash
cargo build --release --target wasm32-wasip2   # produces target/wasm32-wasip2/release/tabtps.wasm
cargo check --target wasm32-wasip2             # fast type-check
cargo clippy --target wasm32-wasip2 -- -D warnings
cargo fmt
```

## Layout

- `src/lib.rs` — `Plugin` trait impl + `pumpkin_plugin_api::register_plugin!` macro export
- `src/join_handler.rs` — `PlayerJoinEvent` handler + per-second tab footer update via `scheduler::schedule_repeating_task`

## Key dependencies

- `pumpkin-plugin-api` (git, branch=master) — `Plugin` trait, `Context`, `Server`, scheduler, event handlers, `TextComponent`
- `tracing` — structured logging via the host server

## Notes

- Crate type is `cdylib` targeting `wasm32-wasip2`; the WIT-based component is loaded by
  the host's wasm plugin loader (`pumpkin/src/plugin/loader/wasm/`).
- `Cargo.toml` pins `pumpkin-plugin-api` to git `master`. Upstream API is unstable —
  expect to chase breaking changes when bumping.
- Color thresholds: green ≤ 25 MSPT, gold > 25 ≤ 40, red > 40.

## Harness

This project uses [`claude-code-harness`](https://github.com/topi-banana/claude-code-harness).
Config is in `harness.toml`; CC plugin scaffold is generated under `.claude-plugin/` by
`harness sync`.
