# pumpkin-tabtps

Pumpkin (Minecraft server framework) plugin that displays live TPS / MSPT in the
player tab list footer. Built as a `cdylib` and dropped into `pumpkin/plugins/`.

## Build

```bash
cargo build --release      # produces target/release/libtabtps.{so,dll,dylib}
cargo check                # fast type-check
cargo clippy -- -D warnings
cargo fmt
```

## Layout

- `src/lib.rs` — `Plugin` trait impl + no-mangle ABI exports (`plugin`, `METADATA`, `PUMPKIN_API_VERSION`)
- `src/join_handler.rs` — player join hook + tab footer update task

## Key dependencies

- `pumpkin*` (git, branch=master) — server, data, protocol, util
- `tokio` (full) — async runtime for the per-second update task
- `log` — structured logging via the host server
- `serde` — config / data serialization

## Notes

- Crate type is `cdylib`; do not rename without updating the loader expectation.
- `Cargo.toml` pins all `pumpkin*` deps to git `master`. Upstream API is unstable —
  expect to chase breaking changes when bumping.
- Color thresholds: green ≤ 25 MSPT, gold > 25 ≤ 40, red > 40.

## Harness

This project uses [`claude-code-harness`](https://github.com/topi-banana/claude-code-harness).
Config is in `harness.toml`; CC plugin scaffold is generated under `.claude-plugin/` by
`harness sync`.
