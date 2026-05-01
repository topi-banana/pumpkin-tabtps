# pumpkin-tabtps

A Pumpkin plugin that displays live server performance stats (TPS and MSPT) in the Minecraft player's tab list footer.

## Overview

This plugin is a Pumpkin-based implementation inspired by [jpenilla/TabTPS](https://github.com/jpenilla/TabTPS), bringing similar tab list performance metrics (TPS/MSPT) to the [Pumpkin](https://github.com/Pumpkin-MC/Pumpkin) server framework.

`pumpkin-tabtps` is a plugin for the Pumpkin Minecraft server framework. It updates the tab list footer every second with real-time TPS (ticks per second) and MSPT (milliseconds per tick) readings, color-coded based on server performance.

- ✅ Displays live TPS and MSPT
- ✅ Color-coded for clarity (green, gold, red)
- ✅ Updates every second
- ✅ Lightweight and asynchronous

## Example Output

![image](https://github.com/user-attachments/assets/4a895184-32df-4f54-b55e-b8b5bb95d65d)

Colors:
- 🟢 **Green**: MSPT ≤ 25
- 🟠 **Gold**: MSPT > 25 and ≤ 40
- 🔴 **Red**: MSPT > 40

## Project Structure

```

pumpkin-tabtps
├── Cargo.toml       # Rust package manifest
├── LICENSE          # MIT License
└── src
    └── lib.rs       # Plugin implementation

````

## Usage

1. **Build the Plugin**
    ```bash
    cargo build --release
    ```

2. **Deploy**

    Copy the resulting `.so`, `.dll`, or `.dylib` file from `target/release/` to your Pumpkin server's plugin directory.

3. **Run Server**

    Start your Pumpkin server, and the plugin will:

    * Log `Hello, Pumpkin!` on load
    * Start updating the tab footer every second when a player joins

## Dependencies

* [`pumpkin`](https://github.com/Pumpkin-MC/Pumpkin) (`master` branch — `pumpkin`, `pumpkin-data`, `pumpkin-protocol`, `pumpkin-util`)
* [`tokio`](https://tokio.rs/) for async runtime
* [`log`](https://docs.rs/log/)

## License

This project is licensed under the MIT License. See [`LICENSE`](./LICENSE) for details.
